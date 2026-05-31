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

    Ok(())
}

fn hex_prefix(bytes: &[u8], len: usize) -> String {
    bytes
        .iter()
        .take(len)
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}
