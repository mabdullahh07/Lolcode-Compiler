//! lolcompiler — compiles a LOLCODE Markdown (.lol) file into HTML and
//! opens the result in the default web browser.
//!
//! Usage: `lolcompiler input.lol`

mod ast;
mod compiler;
mod html;
mod lexer;
mod parser;

use compiler::{Compiler, LolCompiler};
use std::path::Path;
use std::process;

/// Print a usage error and quit.
fn usage_error(message: &str) -> ! {
    eprintln!("Usage error: {}", message);
    eprintln!("Usage: lolcompiler <input.lol>");
    process::exit(1);
}

/// Open the generated HTML file in the system's default browser.
fn open_in_browser(path: &Path) {
    #[cfg(target_os = "windows")]
    let result = process::Command::new("cmd").args(["/C", "start", ""]).arg(path).spawn();
    #[cfg(target_os = "macos")]
    let result = process::Command::new("open").arg(path).spawn();
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let result = process::Command::new("xdg-open").arg(path).spawn();

    if let Err(e) = result {
        eprintln!("Warning: could not open browser: {}", e);
    }
}

fn main() {
    // Exactly one argument, and it must be a .lol file.
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        usage_error("expected exactly one input file");
    }
    let input = Path::new(&args[1]);
    let is_lol = input
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case("lol"))
        .unwrap_or(false);
    if !is_lol {
        usage_error(&format!("'{}' does not have a .lol extension", args[1]));
    }

    let source = std::fs::read_to_string(input).unwrap_or_else(|e| {
        usage_error(&format!("could not read '{}': {}", args[1], e));
    });

    // Compile; any lexical/syntax/semantic error exits inside compile(),
    // so no HTML file is written on failure.
    let mut compiler = LolCompiler::new();
    compiler.compile(&source);
    let html = compiler.html.expect("compile succeeded");

    // Same name and folder as the input, with a .html extension.
    let output = input.with_extension("html");
    if let Err(e) = std::fs::write(&output, html) {
        eprintln!("Error: could not write '{}': {}", output.display(), e);
        process::exit(1);
    }
    println!("Compiled successfully: {}", output.display());

    let absolute = std::fs::canonicalize(&output).unwrap_or(output);
    open_in_browser(&absolute);
}
