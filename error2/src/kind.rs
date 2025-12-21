pub enum ErrorKind<E, B> {
    Std { source: E, backtrace: B },
    Err2 { source: E },
}
