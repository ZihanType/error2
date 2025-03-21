use std::error::Error;

use crate::ErrorExt;

pub enum NextError<'a> {
    Ext(&'a dyn ErrorExt),
    Std(&'a dyn Error),
    None,
}
