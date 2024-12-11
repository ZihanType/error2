use std::error::Error;

use crate::{Location, NextError};

pub trait ErrorExt: Error {
    fn entry(&self) -> (Location, NextError<'_>);

    fn error_stack(&self) -> Box<[Box<str>]> {
        let mut stack = Vec::new();

        let mut next = {
            let (location, next_error) = self.entry();
            stack.push(format!("0: {self}, at {location}").into_boxed_str());
            next_error
        };

        loop {
            let idx = stack.len();

            match next {
                NextError::Ext(e) => {
                    next = {
                        let (location, next_error) = e.entry();
                        stack.push(format!("{idx}: {e}, at {location}").into_boxed_str());
                        next_error
                    };
                    continue;
                }
                NextError::Std(e) => {
                    stack.push(format!("{idx}: {e}").into_boxed_str());
                    break;
                }
                NextError::None => break,
            }
        }

        stack.into_boxed_slice()
    }
}
