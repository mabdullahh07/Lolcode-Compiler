# LOLCODE Markdown Compiler

## Overview

This project is a LOLCODE Markdown Compiler written in Rust. It takes source files written in LOLCODE Markdown, checks their syntax, and converts them into HTML files that can be opened in a web browser.

The project was developed as part of COSC 455.

## Features

- Lexical analysis to convert source code into tokens.
- Syntax analysis to check the structure of the program.
- Abstract Syntax Tree (AST) representation of the document.
- HTML generation from the AST.
- Support for document titles and comments.
- Support for paragraphs and nested paragraphs.
- Support for bold text.
- Support for hyperlinks.
- Support for unordered lists and list items.
- Support for variable definitions and variable usage.
- Variable scopes inside paragraphs and list items.
- Error handling for invalid syntax and undefined variables.
- Automatic generation of HTML output.

## Technologies Used

- Rust
- Cargo
- HTML

## Project Structure

```text
Lolcode-Compiler/
├── src/
│   ├── main.rs
│   ├── lexer.rs
│   ├── parser.rs
│   ├── ast.rs
│   ├── compiler.rs
│   └── html.rs
├── test/
│   ├── Test1.lol
│   ├── Test2.lol
│   └── ...
├── Cargo.toml
└── README.md
```

## Requirements

- Rust
- Cargo

Install Rust from:

https://www.rust-lang.org/tools/install

## How to Run

### 1. Clone or download the project

Open the project folder in a terminal.

### 2. Build the project

```bash
cargo build
```

### 3. Run the compiler

```bash
cargo run -- test/Test1.lol
```

Replace `Test1.lol` with the source file you want to compile.

### 4. View the output

The compiler generates an HTML file with the same name as the source file.

For example:

```text
Test1.lol → Test1.html
```

The generated HTML file is automatically opened in the browser.

## Running Tests

Run the unit tests using:

```bash
cargo test
```

## Example

### LOLCODE Markdown Input

```text
#HAI
#MAEK HEAD
#GIMMEH TITLE My Document #OIC
#MKAY
#MAEK PARAGRAF
Hello world!
#MKAY
#KBYE
```

### Generated HTML

```html
<html>
<head>
<title>My Document</title>
</head>
<body>
<p>Hello world!</p>
</body>
</html>
```

## Error Handling

The compiler reports errors when it encounters invalid annotations, missing required annotations, invalid syntax, or undefined variables.

## Author

Muhammad Abdullah Zia

Towson University  
Computer Science
