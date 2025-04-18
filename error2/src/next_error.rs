use std::error::Error;

use crate::Error2;

pub enum NextError<'a> {
    Err2(&'a dyn Error2),
    Std(&'a dyn Error),
    None,
}
