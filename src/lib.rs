pub mod ast;
mod criterion;
mod debug;
pub mod lexer;
pub mod literals;
pub mod output;
pub mod parser;

use std::path::Path;
use crate::ast::ast::Item;
use crate::lexer::token;
use crate::lexer::token::SpannedToken;
use crate::output::output::Output;
use crate::parser::parser::Parser;

pub fn run(file_path: &Path, options: &ProgramOptions) -> Result<(), String> {
    let source: &str = &*std::fs::read_to_string(file_path)
        .or_else(|e| Err(format!("Failed to read source file: {}", e)))?;
    options.print_source(source, file_path);

    let tokens: Vec<SpannedToken> =
        token::lex(source).or_else(|e| Err(format!("Lexing error: {}", e)))?;
    options.print_tokens(&tokens, file_path);

    let translation: Vec<Item> = Parser::new(tokens).parse_translation_unit().map_err(|e| {
        e.print_span(source);
        format!("{}", e)
    })?;

    let dot: String = debug::dot::dump_translation_unit(&translation);
    options.print_ast(&dot, file_path);

    let rust = (&translation).as_rust_repr();
    options.print_rust(&rust, file_path);

    let c = (&translation).as_c_repr();
    options.print_c(&c, file_path);
    Ok(())
}

/// Output destination for compiler debug information
#[derive(Debug, Clone)]
pub enum OutputTarget {
    Stdout,
    File(String),
}

impl OutputTarget {
    fn write(&self, content: &str, filename: &str) {
        match self {
            OutputTarget::Stdout => println!("{}", content),
            OutputTarget::File(dir_name) => {
                std::fs::write(format!("{}/{}", dir_name, filename), content).expect("Failed to write to file");
            }
        }
    }
}

/// Command-line arguments and options for the compiler
#[derive(Debug, Clone)]
pub struct ProgramOptions {
    /// Input files to process
    pub input_files: Vec<String>,
    /// Where to dump lexer tokens (None = don't dump)
    pub dump_tokens: Option<OutputTarget>,
    /// Where to dump AST in dot format (None = don't dump)
    pub dump_ast: Option<OutputTarget>,
    /// Where to dump Rust output (default: stdout)
    pub dump_rust: Option<OutputTarget>,
    /// Where to dump C output (AST parsed back to C)
    pub dump_c: Option<OutputTarget>,
    /// Where to dump source code
    pub dump_source: Option<OutputTarget>,
    /// Optional output directory for file-based outputs
    pub output_dir: Option<String>,
}

impl ProgramOptions {
    /// Parse command-line arguments
    ///
    /// Usage: program [OPTIONS] [FILES...]
    ///
    /// Options:
    ///   --dump-tokens    Output lexer tokens
    ///   --dump-ast       Output AST in dot format
    ///   --dump-rust      Output Rust translation (default)
    ///   --dump-c         Output C code (AST parsed back to C)
    ///   --dump-source    Output original source code
    ///   --no-dump-rust   Disable Rust output
    ///   -o, --output-dir Write outputs to directory instead of stdout
    ///   -h, --help       Show this help message
    pub fn parse(args: std::env::Args) -> Result<Self, String> {
        let mut input_files = Vec::new();
        let mut dump_tokens = None;
        let mut dump_ast = None;
        let mut dump_rust = Some(OutputTarget::Stdout); // default
        let mut dump_c = None;
        let mut dump_source = None;
        let mut output_dir: Option<String> = None;

        let mut args_iter = args.skip(1); // skip program name

        while let Some(arg) = args_iter.next() {
            match arg.as_str() {
                "--dump-tokens" => {
                    dump_tokens = Some(OutputTarget::Stdout);
                }
                "--dump-ast" => {
                    dump_ast = Some(OutputTarget::Stdout);
                }
                "--dump-rust" => {
                    dump_rust = Some(OutputTarget::Stdout);
                }
                "--dump-c" => {
                    dump_c = Some(OutputTarget::Stdout);
                }
                "--dump-source" => {
                    dump_source = Some(OutputTarget::Stdout);
                }
                "--no-dump-rust" => {
                    dump_rust = None;
                }
                "-o" | "--output-dir" => {
                    output_dir = args_iter
                        .next()
                        .ok_or_else(|| "Expected directory path after -o/--output-dir".to_string())
                        .ok();
                }
                "-h" | "--help" => {
                    return Err("help".to_string()); // Signal to print help
                }
                arg if arg == "--" => {
                    args_iter.next();
                    args_iter.for_each(|a| input_files.push(a.to_string()));
                    break;
                }
                arg if arg.starts_with('-') => {
                    return Err(format!("Unknown option: {}", arg));
                }
                _ => {
                    input_files.push(arg);
                }
            }
        }

        if input_files.is_empty() {
            return Err("No input files specified".to_string());
        }

        // If output_dir is set, convert Stdout targets to File targets
        let (dump_tokens, dump_ast, dump_rust, dump_c, dump_source) = if let Some(ref dir) =
            output_dir
        {
            (
                dump_tokens.map(|_| OutputTarget::File(format!("{}", dir))),
                dump_ast.map(|_| OutputTarget::File(format!("{}", dir))),
                dump_rust.map(|_| OutputTarget::File(format!("{}", dir))),
                dump_c.map(|_| OutputTarget::File(format!("{}", dir))),
                dump_source.map(|_| OutputTarget::File(format!("{}", dir))),
            )
        } else {
            (dump_tokens, dump_ast, dump_rust, dump_c, dump_source)
        };

        Ok(ProgramOptions {
            input_files,
            dump_tokens,
            dump_ast,
            dump_rust,
            dump_c,
            dump_source,
            output_dir,
        })
    }

    /// Print output if target is set
    pub fn print_tokens(&self, tokens: &Vec<SpannedToken>, path: &Path) {
        if let Some(ref target) = self.dump_tokens {
            let formatted = tokens
                .iter()
                .map(|t| format!("  {:?}  @ {}..{}", t.token, t.span.start, t.span.end))
                .collect::<Vec<_>>()
                .join("\n");
            target.write(&formatted, &format!("{}.tokens", stem_or_debug!(path)));
        }
    }

    pub fn print_ast(&self, dot: &str, path : &Path) {
        if let Some(ref target) = self.dump_ast {
            target.write(dot, &format!("{}.ast.dot", stem_or_debug!(path)));
        }
    }

    pub fn print_rust(&self, rust: &str, path: &Path) {
        if let Some(ref target) = self.dump_rust {
            target.write(rust, &format!("{}.debug.rs", stem_or_debug!(path)));
        }
    }

    pub fn print_c(&self, c: &str, path: &Path) {
        if let Some(ref target) = self.dump_c {
            target.write(c, &format!("{}.debug.c", stem_or_debug!(path)));
        }
    }

    pub fn print_source(&self, source: &str, path: &Path) {
        if let Some(ref target) = self.dump_source {
            target.write(source, &format!("{}.src.c", stem_or_debug!(path)));
        }
    }

    pub fn print_help() {
        eprintln!(
            r#"Crustpiler: C to Rust compiler

Usage: crustpiler [OPTIONS] <FILES>...

Arguments:
  <FILES>...              Input C files to process

Options:
  --dump-tokens           Output lexer tokens to stdout
  --dump-ast              Output AST in dot format to stdout
  --dump-source           Output original source code to stdout
  --dump-rust             Output Rust translation (enabled by default)
  --dump-c                Output C code (AST parsed back to C)
  --no-dump-rust          Disable Rust output
  -o, --output-dir DIR    Write all outputs to directory instead of stdout
  -h, --help              Print help message
"#
        );
    }
}

#[macro_export(local_inner_macros)]
macro_rules! stem_or_debug {
    ($path:expr) => {
        $path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("debug")
    };
}