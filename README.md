# Error2

[![Crates.io version](https://img.shields.io/crates/v/error2.svg?style=flat-square)](https://crates.io/crates/error2)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/error2)

`Error2` is a trait that extends the `std::error::Error` trait with additional methods.

Its definition is:

```rust
pub trait Error2: Error {
    fn backtrace(&self) -> &Backtrace;
    fn backtrace_mut(&mut self) -> &mut Backtrace;
}
```

## Example

```rust
use std::io;

use error2::{Attach, Backtrace, Error2, Context};

#[derive(Debug, Error2)]
pub enum CustomError {
    #[error2(display("IO error"))]
    IoError {
        source: io::Error,
        backtrace: Backtrace,
    },

    #[error2(display("User not found: {username}"))]
    NotFoundUser {
        username: String,
        backtrace: Backtrace,
    },
}

fn read_file() -> Result<Vec<u8>, CustomError> {
    std::fs::read("aaaaa.txt").context(IoError2)
}

fn main() {
    let result = read_file().attach();

    if let Err(e) = result {
        // Print the error message:
        //
        // test_error2::CustomError: IO error
        //     at src/main.rs:26:30
        //     at src/main.rs:22:32
        // std::io::error::Error: No such file or directory (os error 2)
        println!("{}", error2::extract_error_message(&e));
    }
}
```
