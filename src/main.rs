// =============================================================================
// main.rs — pipeline: source → logos lexer → recursive-descent parser → output
// =============================================================================
fn process_directory(directory_root: &str) {
    if std::path::Path::new(directory_root).is_file() {
        println!("Processing file: {}", directory_root.to_string());
        crustpiler::run(directory_root.to_string())
            .map_err(|e| {
                eprintln!("{}", e);
            })
            .ok();
        return;
    }
    println!("directory_root:{}", directory_root);
    for entry in std::fs::read_dir(directory_root).expect("Failed to read directory.") {
        let entry = entry.expect("Failed to read directory entry.");
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .map(|s| s == "c" || s == "h")
                .unwrap_or(false)
        {
            println!("Processing file: {}", path.to_str().unwrap());
            crustpiler::run(path.to_str().unwrap().to_string())
                .map_err(|e| {
                    eprintln!("{}", e);
                })
                .unwrap();
        } else if path.is_dir() && !path.is_symlink() {
            process_directory(path.to_str().unwrap());
        }
    }
}

fn main() {
    if std::env::args().len() < 2 {
        eprintln!("Usage: criterion-to-rust <directory-root>");
        std::process::exit(1);
    }
    std::env::args()
        .skip(1)
        .for_each(|arg| process_directory(&arg));
}
