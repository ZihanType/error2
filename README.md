# Error2

English | [简体中文](README-zh_cn.md)

[![Crates.io version](https://img.shields.io/crates/v/error2.svg?style=flat-square)](https://crates.io/crates/error2)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/error2)

Error handling library with backtrace support and ergonomic error conversion.

## Installation

```toml
[dependencies]
error2 = "0.13.2"
```

## Usage

### Define Error Types

Error2 supports three types of errors based on field structure:

**1. Root Error** - Only `backtrace` field (new error source)

```rust
use error2::prelude::*;

#[derive(Debug, Error2)]
#[error2(display("User {username} not found"))]
struct UserNotFound {
    username: String,
    backtrace: Backtrace,
}
```

**2. Standard Error** - `source` + `backtrace` (wraps `std::error::Error`)

```rust
use error2::prelude::*;

#[derive(Debug, Error2)]
#[error2(display("IO error: {path}"))]
struct IoError {
    path: String,
    source: std::io::Error,
    backtrace: Backtrace,
}
```

**3. Error2 Chain** - Only `source` field (chains another `Error2` type, reuses its backtrace)

```rust
use error2::prelude::*;

#[derive(Debug, Error2)]
#[error2(display("Application error"))]
struct AppError {
    source: IoError,  // Another Error2 type
}
```

### Convert Errors

```rust
// Result<T, Source> -> Result<T, Target>
std::fs::read_to_string("file.txt")
    .context(IoError2 { path: "file.txt" })?;

// Option<T> -> Result<T, E>
database.get(id)
    .context(NotFound2 { item: "user" })?;

// E -> Result<T, E>
some_error.context(MyError2)
```

**Note:** Helper struct fields are generic with `Into` trait bounds, so you don't need to call `.into()` manually. For expensive conversions, use `.with_context()` instead:

```rust
// Lazy evaluation - only converts on error
std::fs::read(&path)
    .with_context(|| IoError2 { path: path.display().to_string() })?;
```

### Track Propagation

```rust
fn inner() -> Result<(), MyError> {
    operation().context(IoError2 { path: "file.txt" })?
}

fn outer() -> Result<(), MyError> {
    inner().attach()?  // Captures this location
}
```

### Create Root Errors

```rust
fn validate(value: i32) -> Result<(), ValidationError> {
    if value < 0 {
        return ValidationError2 { value }.fail();
    }
    Ok(())
}
```

### Use Type-Erased Errors

```rust
fn operation() -> Result<(), BoxedError2> {
    std::fs::read("file.txt").context(ViaStd)?;

    if condition {
        ViaRoot("error message").fail()?;
    }

    other_operation().context(ViaErr2)?;

    Ok(())
}
```

### Print Error Stack

```rust
fn main() {
    match operation().attach() {
        Ok(_) => println!("Success"),
        Err(e) => {
            // Print complete error chain with locations:
            //
            // IoError: io error: config.txt
            //     at src/main.rs:15:42
            //     at src/main.rs:23:5
            // std::io::Error: No such file or directory (os error 2)
            eprintln!("{}", e.backtrace().error_message());
        }
    }
}
```

For more details, see [API Documentation](https://docs.rs/error2)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
