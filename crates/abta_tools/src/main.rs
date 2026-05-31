use std::path::PathBuf;

fn main() {
    let mut arguments = std::env::args_os().skip(1);
    let command = arguments.next();

    match command {
        Some(arg) if arg == "--help" || arg == "-h" => {
            print_help();
        }
        Some(arg) if arg == "inspect" => {
            let Some(path) = arguments.next().map(PathBuf::from) else {
                eprintln!("Usage: abta_tools inspect <path-to-TA.EPF>");
                std::process::exit(2);
            };

            if let Err(error) = inspect_archive(path) {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
        Some(arg) if arg == "list" => {
            let Some(path) = arguments.next().map(PathBuf::from) else {
                eprintln!("Usage: abta_tools list <path-to-TA.EPF> [--ext EXT]");
                std::process::exit(2);
            };
            let filter = match parse_list_filter(arguments.collect()) {
                Ok(filter) => filter,
                Err(error) => {
                    eprintln!("error: {error}");
                    std::process::exit(2);
                }
            };

            if let Err(error) = list_archive(path, filter) {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
        Some(arg) => {
            let path = PathBuf::from(arg);

            if let Err(error) = inspect_archive(path) {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
        None => {
            print_help();
        }
    }
}

fn print_help() {
    println!("Usage:");
    println!("  abta_tools inspect <path-to-TA.EPF>");
    println!("  abta_tools list <path-to-TA.EPF> [--ext EXT]");
    println!("  abta_tools <path-to-TA.EPF>");
    println!();
    println!("Local-only research tooling for ABTA resource inspection.");
}

fn inspect_archive(path: PathBuf) -> Result<(), String> {
    let bytes = std::fs::read(&path).map_err(|error| format!("failed to read file: {error}"))?;

    if bytes.len() < 8 {
        return Err("file is too small to be an EPF archive".to_string());
    }

    let magic = &bytes[0..4];
    let header_value = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let directory_offset = header_value as usize;

    println!("archive: {}", path.display());
    println!("size_bytes: {}", bytes.len());
    println!("magic: {}", String::from_utf8_lossy(magic));
    println!("header_value_le_u32: {header_value}");

    if magic == b"EPFS" {
        println!("format_hint: EPFS archive");
    } else {
        println!("format_hint: unknown");
    }

    println!("first_16_bytes_hex: {}", hex_prefix(&bytes, 16));

    if directory_offset < bytes.len() {
        let entries = parse_directory(&bytes, directory_offset)?;
        println!("directory_offset: {directory_offset}");
        println!("directory_size_bytes: {}", bytes.len() - directory_offset);
        println!("directory_entries: {}", entries.len());
        println!("directory_extensions: {}", extension_summary(&entries));
        println!("directory_parse: heuristic_partial");
    }

    Ok(())
}

fn list_archive(path: PathBuf, filter: ListFilter) -> Result<(), String> {
    let bytes = std::fs::read(&path).map_err(|error| format!("failed to read file: {error}"))?;

    if bytes.len() < 8 {
        return Err("file is too small to be an EPF archive".to_string());
    }

    let magic = &bytes[0..4];

    if magic != b"EPFS" {
        return Err("file does not start with EPFS magic".to_string());
    }

    let directory_offset = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    let entries = parse_directory(&bytes, directory_offset)?;
    let filtered_entries = filter_entries(&entries, &filter);

    println!("archive: {}", path.display());
    println!("entries: {}", entries.len());
    println!("listed_entries: {}", filtered_entries.len());
    println!("extensions: {}", extension_summary(&entries));
    println!("parse: heuristic_partial");
    println!();
    println!("name\tmeta_hex\toffset\tsize");

    for entry in filtered_entries {
        println!(
            "{}\t{}\t{}\t{}",
            entry.name,
            hex_prefix(&entry.metadata, entry.metadata.len()),
            entry.offset,
            entry.size
        );
    }

    Ok(())
}

fn parse_list_filter(args: Vec<std::ffi::OsString>) -> Result<ListFilter, String> {
    let mut filter = ListFilter::default();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        if arg == "--ext" {
            let Some(extension) = args.next() else {
                return Err("--ext requires a value".to_string());
            };

            filter.extension = Some(
                extension
                    .to_string_lossy()
                    .trim_start_matches('.')
                    .to_ascii_uppercase(),
            );
        } else {
            return Err(format!("unknown list option: {}", arg.to_string_lossy()));
        }
    }

    Ok(filter)
}

fn hex_prefix(bytes: &[u8], len: usize) -> String {
    bytes
        .iter()
        .take(len)
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug)]
struct EpfEntry {
    name: String,
    metadata: Vec<u8>,
    offset: u32,
    size: u32,
}

#[derive(Debug, Default)]
struct ListFilter {
    extension: Option<String>,
}

fn filter_entries<'a>(entries: &'a [EpfEntry], filter: &ListFilter) -> Vec<&'a EpfEntry> {
    entries
        .iter()
        .filter(|entry| {
            let Some(extension) = &filter.extension else {
                return true;
            };

            entry_extension(&entry.name).as_deref() == Some(extension.as_str())
        })
        .collect()
}

fn parse_directory(bytes: &[u8], directory_offset: usize) -> Result<Vec<EpfEntry>, String> {
    if directory_offset >= bytes.len() {
        return Err(format!(
            "directory offset {directory_offset} is outside file of {} bytes",
            bytes.len()
        ));
    }

    let mut cursor = directory_offset;
    let mut entries = Vec::new();

    while cursor < bytes.len() {
        let name_start = cursor;

        while cursor < bytes.len() && bytes[cursor] != 0 {
            cursor += 1;
        }

        if cursor == bytes.len() {
            break;
        }

        if cursor == name_start {
            return Err(format!("empty entry name at directory byte {cursor}"));
        }

        let name = String::from_utf8_lossy(&bytes[name_start..cursor]).to_string();
        cursor += 1;

        let Some((metadata, offset, size, consumed)) =
            parse_entry_tail(bytes, cursor, directory_offset)
        else {
            break;
        };
        cursor += consumed;

        entries.push(EpfEntry {
            name,
            metadata,
            offset,
            size,
        });
    }

    Ok(entries)
}

fn parse_entry_tail(
    bytes: &[u8],
    cursor: usize,
    directory_offset: usize,
) -> Option<(Vec<u8>, u32, u32, usize)> {
    for metadata_len in 1..=8 {
        if cursor + metadata_len + 8 > bytes.len() {
            return None;
        }

        let offset_start = cursor + metadata_len;
        let size_start = offset_start + 4;
        let offset = u32::from_le_bytes([
            bytes[offset_start],
            bytes[offset_start + 1],
            bytes[offset_start + 2],
            bytes[offset_start + 3],
        ]);
        let size = u32::from_le_bytes([
            bytes[size_start],
            bytes[size_start + 1],
            bytes[size_start + 2],
            bytes[size_start + 3],
        ]);

        let data_end = offset as usize + size as usize;

        let next_cursor = cursor + metadata_len + 8;
        let next_entry_starts_cleanly =
            next_cursor == bytes.len() || bytes[next_cursor].is_ascii_graphic();

        if offset >= 8 && size > 0 && data_end <= directory_offset && next_entry_starts_cleanly {
            return Some((
                bytes[cursor..cursor + metadata_len].to_vec(),
                offset,
                size,
                metadata_len + 8,
            ));
        }
    }

    None
}

fn extension_summary(entries: &[EpfEntry]) -> String {
    let mut extensions = std::collections::BTreeMap::<String, usize>::new();

    for entry in entries {
        let extension = entry_extension(&entry.name).unwrap_or_else(|| "<NONE>".to_string());
        *extensions.entry(extension).or_default() += 1;
    }

    extensions
        .into_iter()
        .map(|(extension, count)| format!("{extension}:{count}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn entry_extension(name: &str) -> Option<String> {
    name.rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_uppercase())
}
