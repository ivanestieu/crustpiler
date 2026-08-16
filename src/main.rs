// =============================================================================
// main.rs — pipeline: source → logos lexer → recursive-descent parser → output
// =============================================================================

/// Process a file or directory (recursively)
fn process_path(path: &str, options: &crustpiler::ProgramOptions) {
    let path_obj = std::path::Path::new(path);

    if path_obj.is_file() {
        println!("Processing file: {}", path);
        if let Err(e) = crustpiler::run(path_obj, options) {
            eprintln!("Error processing {}: {}", path, e);
        }
        return;
    }

    /* if path_obj.is_dir() {
        println!("Processing directory: {}", path);
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let entry_path = entry.path();
                    if entry_path.is_file()
                        && entry_path
                            .extension()
                            .map(|s| s == "c" || s == "h")
                            .unwrap_or(false)
                    {
                        process_path(entry_path.to_str().unwrap(), options);
                    } else if entry_path.is_dir() && !entry_path.is_symlink() {
                        process_path(entry_path.to_str().unwrap(), options);
                    }
                }
            }
        }
        return;
    }*/

    eprintln!("Path not found: {}", path);
}

fn main() {
    let args = std::env::args();

    let options = match crustpiler::ProgramOptions::parse(args) {
        Ok(opts) => opts,
        Err(msg) => {
            if msg == "help" {
                crustpiler::ProgramOptions::print_help();
            } else {
                eprintln!("Error: {}", msg);
                crustpiler::ProgramOptions::print_help();
            }
            std::process::exit(1);
        }
    };

    if let Some(ref dir) = options.output_dir {
        if let Err(e) = std::fs::create_dir_all(dir) {
            eprintln!("Failed to create output directory '{}': {}", dir, e);
            std::process::exit(1);
        }
    }

    for input in &options.input_files {
        process_path(input, &options);
    }
}
