# Crustpiler

A C11 to Rust transpiler built as a comprehensive learning project for systems programming and embedded systems development.

## Overview

**Crustpiler** is a C11 to Rust transpiler project, currently in active development. The goal is to build systematic tooling that converts C source code into Rust, helping bridge legacy C codebases to modern, memory-safe Rust while serving as a comprehensive learning vehicle.

### Why Crustpiler?

This project started from a practical need: translating criterion test suites from C to Rust to gain hands-on experience with Rust. Rather than manual translation, I started with a very minimal python script, but as I was facing growing issues I decided to build a transpiler, using it as my primary vehicle for learning Rust.
As I am experienced in compiler development, it was an accessible project.

**Rust for embedded systems** ranks as the third most adopted language in the embedded industry (after C and C++). By building Crustpiler, I'm:
- Mastering Rust through real compiler/transpiler implementation
- Gaining deep understanding of language translation and AST construction
- Creating tools that could eventually modernize legacy C codebases
- Learning the patterns and trade-offs in C-to-Rust semantic translation

## Current Status

This is an **early-stage project** in active development. The foundation is being built systematically:

Completed:
- **Lexical Analysis**: Complete C11 tokenization using the `logos` crate
- **Batch Processing**: Process single files or entire directory trees with one command
- **Debug Output**: Generates `.dot` files for AST visualization using Graphviz

Work in progress:
- **Error Reporting**: Reports lexing errors with position information
- **Parser**: Recursive descent parser in progress (declarations partially tested, definitions not yet implemented)

Partially working, needing evolution:
- **AST Construction**: evolving with bug discoveries
- **Basic Type Rewriting**: Simple C-to-Rust type mappings (no pattern matching yet)
- **Code Generation**: Early stage; currently basic type rewriting without full transpilation

## Architecture

```
┌──────────────────┐
│   C11 Source     │
└────────┬─────────┘
         │
         ▼
┌──────────────────────────────┐
│  Lexer (logos crate)         │  Tokenization with position tracking
└────────┬─────────────────────┘
         │
         ▼
┌──────────────────────────────┐
│  Recursive Descent Parser    │  AST construction
└────────┬─────────────────────┘
         │
         ▼
┌──────────────────────────────┐
│  Abstract Syntax Tree (AST)  │  Intermediate representation
└────────┬─────────────────────┘
         │
         ├─► Debug Output (.dot)
         │
         ▼
┌──────────────────────────────┐
│  Rust Code Generator         │  Idiomatic Rust emission
└────────┬─────────────────────┘
         │
         ▼
┌──────────────────┐
│  Rust Source     │
└──────────────────┘
```

### Key Components

- **`lexer/`**: Tokenization using the `logos` crate for efficient lexical analysis
- **`parser/`**: Recursive descent parser implementing C11 grammar
- **`ast/`**: AST node definitions and traversal utilities
- **`output/`**: Rust code generation from the AST
- **`literals/`**: Support for C numeric and string literal parsing
- **`debug/`**: DOT format output for AST visualization

## Getting Started

### Prerequisites

- Rust 1.70+ (2024 edition)
- Cargo

### Installation

```bash
git clone https://github.com/ivanestieu/crustpiler.git
cd crustpiler
cargo build --release
```

### Usage

#### Transpile a single C file

```bash
cargo run --release -- path/to/file.c
```

The transpiler will:
1. Read the C source file
2. Print the tokens (with position info)
3. Generate an AST
4. Output the Rust translation to stdout
5. Write an AST visualization to `file.dot`

#### Process an entire directory

```bash
cargo run --release -- path/to/c/codebase/
```

Recursively processes all `.c` and `.h` files in the directory tree.

#### Visualize the AST

After processing, view the generated `.dot` file:

```bash
# Generate PNG from DOT (requires Graphviz)
dot -Tpng file.dot -o file.png
```

## Development

### Project Structure

```
crustpiler/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library orchestration
│   ├── ast/
│   │   └── ast.rs           # AST node definitions
│   ├── lexer/
│   │   ├── mod.rs
│   │   └── token.rs         # Token types and lexer
│   ├── parser/
│   │   ├── mod.rs
│   │   └── parser.rs        # Recursive descent parser
│   ├── output/
│   │   ├── mod.rs
│   │   └── output.rs        # Rust code generation
│   ├── literals.rs          # Literal parsing utilities
│   ├── debug/
│   │   ├── mod.rs
│   │   └── dot.rs           # DOT format output
│   └── criterion.rs         # Criterion-specific utilities
├── tests/                   # Test suite
├── Cargo.toml              # Project manifest
└── README.md               # This file
```

### Running Tests

```bash
cargo test
```

### Building Documentation

```bash
cargo doc --open
```

## Development Focus

The project is systematically building support for C11 constructs in this order:

### Tokenization (Complete)
- Primitive types: `int`, `float`, `double`, `char`, `void`
- Qualifiers: `const`, `static`, `volatile`
- Keywords and operators
- Integer literals (decimal, hex, octal, binary)
- Floating-point and string literals

### Parsing (In Progress)
- **Declarations** (partial): Type specifiers, declarators, basic parameter lists
- **Definitions**: Not yet implemented
- Control flow structures: `if`, `while`, `for`, `switch` (not yet parsed)
- Expressions and operators (foundation being laid)

### Code Generation (Planning Phase)
- Basic type mapping: C primitives → Rust equivalents
- Future: function signatures, control flow translation, pointer-to-reference conversion

## Roadmap

### Phase 1: Foundation (Current)
- Complete lexical analysis (logos-based tokenizer)
- Parser core (recursive descent structure in place)
- AST for declarations (in testing)
- AST for definitions (function bodies, variable initializations)
- Full expression parsing

### Phase 2: Core Transpilation
- Basic type mapping (C primitives → unsafe Rust)
- Function signature translation
- Variable declaration output
- Control flow statement parsing and generation
- Expression transpilation

### Phase 3: Advanced Features
- Pointer-to-reference semantic translation
- Memory safety transformations
- Error recovery and diagnostics
- Macro and preprocessor support
- Optimization passes

## Learning Value

This project is designed as a comprehensive Rust learning experience:

1. **Compiler Fundamentals** (in progress)
   - Lexical analysis with `logos` crate
   - Recursive descent parser construction
   - AST design and traversal
   - Code generation strategies

2. **Rust Language Mastery**
   - Working with advanced crates (`logos`, `itertools`, `hexf-parse`)
   - Error handling patterns
   - Generic programming and trait design
   - Systems programming concepts

3. **Semantic Translation**
   - Understanding C11 semantics deeply
   - Designing Rust equivalent patterns
   - Type system differences and conversions
   - Memory model considerations for embedded systems

## Known Limitations & Future Work

As an early-stage project, several aspects are not yet implemented:

- **Function definitions**: Parser currently handles declarations; function bodies are in progress
- **Pattern matching**: No semantic analysis or pattern-based transformations yet
- **Full transpilation**: Currently performs basic type rewriting; full code generation is planned
- **Memory semantics**: C pointers and Rust ownership require careful semantic translation (future work)
- **Macros**: Preprocessor directives are not in scope for v0.1
- **Error recovery**: Parsing errors halt processing; recovery mechanisms planned
- **Optimization**: No optimization passes yet implemented

## Contributing

This is primarily a personal learning project, but insights and discussions are welcome:

- **Questions**: Open an issue to discuss transpilation strategies
- **Bug reports**: If the transpiler crashes or produces incorrect output, file an issue
- **Ideas**: Suggestions for language features or architecture improvements

## License

This project is published under the **Portfolio Display License v1.0**. You may:
- View and study the source code
- Clone for personal evaluation
- Reference when assessing the author's skills

See [LICENSE.md](LICENSE.md) for full terms. Without prior written permission, you may not reuse this code in other projects or redistributions.

## Contact

**Author**: Ivan Estieu  
**GitHub**: [@ivanestieu](https://github.com/ivanestieu)

---

**Building better software by learning how compilers work.** 🚀
