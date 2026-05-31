use std::path::PathBuf;

fn main() {
    let argument = std::env::args_os().nth(1);

    match argument {
        Some(arg) if arg == "--help" || arg == "-h" => {
            println!("Usage: abta_tools <path-to-TA.EPF>");
            println!("Local-only research tooling placeholder for ABTA resource inspection.");
        }
        Some(arg) => {
            let path = PathBuf::from(arg);

            println!("ABTA research tooling placeholder");
            println!("archive: {}", path.display());
        }
        None => {
            println!("Usage: abta_tools <path-to-TA.EPF>");
        }
    }
}
