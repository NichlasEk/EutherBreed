use std::path::PathBuf;

fn main() {
    let archive_path = std::env::args_os().nth(1).map(PathBuf::from);

    match archive_path {
        Some(path) => {
            println!("ABTA research tooling placeholder");
            println!("archive: {}", path.display());
        }
        None => {
            println!("Usage: abta_tools <path-to-TA.EPF>");
        }
    }
}
