// =============================================================================
// main.rs — pipeline: source → logos lexer → recursive-descent parser → output
// =============================================================================
fn process_directory(directory_root: &str) {
    for entry in std::fs::read_dir(directory_root).expect("Failed to read directory.") {
        let entry = entry.expect("Failed to read directory entry.");
        let path = entry.path();
        if path.is_file() && path.extension().map(|s| s == "c").unwrap_or(false) {
            println!("Processing file: {:?}", path);
            criterion_to_rust::run(path.to_str().unwrap().to_string());
        }
        else if path.is_dir() && !path.is_symlink() {
            process_directory(path.to_str().unwrap());
        }
    }
}

fn main() {
    if std::env::args().len() < 2 {
        eprintln!("Usage: criterion-to-rust <directory-root>");
        std::process::exit(1);
    }
    let directory_root = std::env::args().nth(1).unwrap();
    process_directory(&*directory_root);
}

