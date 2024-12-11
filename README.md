# Error2

[![Crates.io version](https://img.shields.io/crates/v/error2.svg?style=flat-square)](https://crates.io/crates/error2)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/error2)

`ErrorExt` is a trait that extends the `std::error::Error` trait with additional methods.

It defines two methods:
- `fn entry(&self) -> (Location, NextError<'_>)`, a required method, the implementer needs to return the location of the current error and the next error.
- `fn error_stack(&self) -> Box<[Box<str>]>`, a provided method, will return the stack information of the current error.

## Example

```rust
use error2::{ErrorExt, Location, NextError};
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
#[snafu(display("IO error"))]
pub struct IoError {
    #[snafu(implicit)]
    location: Location,
    source: std::io::Error,
}

impl ErrorExt for IoError {
    fn entry(&self) -> (Location, NextError<'_>) {
        (self.location, NextError::Std(&self.source))
    }
}

fn main() {
    let result = std::fs::read("aaaaa.txt").context(IoSnafu);

    if let Err(e) = result {
        // Print the error stack
        // [
        //     "0: IO error, at src/main.rs:19:45",
        //     "1: No such file or directory (os error 2)",
        // ]
        println!("{:#?}", e.error_stack());
    }
}
```
