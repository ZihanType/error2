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
use std::{error, fmt, io};

use error2::{Attach, Backtrace, Error2, ResultExt};

#[derive(Debug, Error2)]
#[error2(display("IO error"))]
pub struct IoErrorWrapper {
    source: io::Error,
    backtrace: Backtrace,
}

#[derive(Debug, Error2)]
pub enum Error<T, U, S>
where
    T: Error2 + 'static,
    U: fmt::Display + fmt::Debug,
    S: error::Error + 'static,
{
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

    #[error2(display("An error occurred, {some_field}"))]
    OtherStd {
        some_field: U,
        source: S,
        backtrace: Backtrace,
    },

    #[error2(display("An error occurred, {some_field}"))]
    OtherErr2 { some_field: U, source: T },
}

fn read_file() -> Result<Vec<u8>, Error<IoErrorWrapper, i32, io::Error>> {
    std::fs::read("aaaaa.txt").context(IoError2)
}

fn main() {
    let result = read_file().attach();

    if let Err(e) = result {
        // Print the error message:
        //
        // test_error2::Error<test_error2::IoErrorWrapper, i32, std::io::error::Error>: IO error
        //     at src/main.rs:50:30
        //     at src/main.rs:46:32
        // std::io::error::Error: No such file or directory (os error 2)
        println!("{}", error2::extract_error_message(&e));
    }
}
```
