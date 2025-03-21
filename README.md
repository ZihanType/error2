# Error2

[![Crates.io version](https://img.shields.io/crates/v/error2.svg?style=flat-square)](https://crates.io/crates/error2)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/error2)

`ErrorExt` is a trait that extends the `std::error::Error` trait with additional methods.

It defines two methods:
- `fn entry(&self) -> (&Locations, NextError<'_>)`, the implementer needs to return the locations of the current error and the next error.
- `fn locations(&mut self) -> &mut Locations`, the implementer needs to return a mutable reference to the locations of the current error.

## Example

```rust
use error2::{Attach, ErrorExt, Locations};
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu, ErrorExt)]
#[snafu(display("IO error"))]
pub struct IoError {
    #[snafu(implicit)]
    locations: Locations,
    #[error2(std)]
    source: std::io::Error,
}

fn read_file() -> Result<Vec<u8>, IoError> {
    std::fs::read("aaaaa.txt").context(IoSnafu)
}

fn main() {
    let result = read_file().attach();

    if let Err(e) = result {
        // Print the error stack
        // [
        //     "0: IO error, at src/main.rs:18:30",
        //     "1: IO error, at src/main.rs:14:32",
        //     "2: No such file or directory (os error 2)",
        // ]
        println!("{:#?}", error2::extract_error_stack(&e));
    }
}
```
