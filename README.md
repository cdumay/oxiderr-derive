# oxiderr-derive

[![License: BSD-3-Clause](https://img.shields.io/badge/license-BSD--3--Clause-blue)](./LICENSE)
[![oxiderr-derive on crates.io](https://img.shields.io/crates/v/oxiderr-derive)](https://crates.io/crates/oxiderr-derive)
[![oxiderr-derive on docs.rs](https://docs.rs/oxiderr-derive/badge.svg)](https://docs.rs/oxiderr-derive)
[![Source Code Repository](https://img.shields.io/badge/Code-On%20GitHub-blue?logo=GitHub)](https://github.com/cdumay/oxiderr-derive)

The `oxiderr-derive` crate provides procedural macros to simplify the creation of custom error types in Rust. By leveraging these macros,
developers can efficiently define error structures that integrate seamlessly with the `oxiderr` error management ecosystem.

## Overview

Error handling in Rust often involves creating complex structs to represent various error kinds and implementing traits to provide context and
conversions. The `oxiderr-derive` crate automates this process by offering macros that generate the necessary boilerplate code, allowing for
more readable and maintainable error definitions.

## Features

* **Macros**: Automatically generate implementations for custom error types.
* **Integration with oxiderr**: Designed to work cohesively with the `oxiderr` crate, ensuring consistent error handling patterns.

## Usage

To utilize `oxiderr-derive` in your project, follow these steps:

1. **Add Dependencies**: Include `oxiderr` with the feature `derive` in your `Cargo.toml`:

```toml
[dependencies]
oxiderr = { version = "0.1", features = ["derive"] }
```

2. **Define Error**: Use the provided derive macros to define your error and error kind structs:

```rust
use oxiderr::{define_errors, define_kinds, AsError};

define_kinds! {
    UnknownError = ("Err-00001", 500, "Unexpected error"),
    IoError = ("Err-00001", 400, "IO error")
}
define_errors! {
    Unexpected = UnknownError,
    FileRead = IoError,
    FileNotExists = IoError
}
```
In this example:

* define_kinds create `oxiderr::ErrorKind` structs representing different categories of errors.
* define_errors create `oxiderr::Error` struct that contains an ErrorKind and metadata.

3. **Implementing Error Handling**: With the above definitions, you can now handle errors in your application as follows:

```rust
use std::fs::File;
use std::io::Read;

fn try_open_file(path: &str) -> oxiderr::Result<String> {
    let mut file = File::open(path).map_err(|err| FileNotExists::new().set_message(err.to_string()))?;
    let mut content = String::new();
    file.read_to_string(&mut content).map_err(|err| FileRead::new().set_message(err.to_string()))?;
    Ok(content)
}

fn main() {
    let path = "example.txt";

    match try_open_file(path) {
        Ok(content) => println!("File content:\n{}", content),
        Err(e) => eprintln!("{}", e),
    }
}
```
This will output:

```
[Err-00001] Client::IoError::FileNotExists (400) - No such file or directory (os error 2)
```
