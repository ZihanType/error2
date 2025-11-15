/// Constructs a [`Location`](crate::Location) that is not affected by `#[track_caller]`.
#[macro_export]
macro_rules! location {
    () => {
        $crate::Location::new(::core::file!(), ::core::line!(), ::core::column!())
    };
}
