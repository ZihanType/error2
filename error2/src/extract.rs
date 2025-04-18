use crate::{Error2, NextError};

pub fn extract_error_stack(e: &dyn Error2) -> Box<[Box<str>]> {
    fn extract_single<'a>(stack: &mut Vec<Box<str>>, e: &'a dyn Error2) -> NextError<'a> {
        let (locations, next_error) = e.entry();

        for location in locations.inner().iter().rev() {
            let idx = stack.len();
            stack.push(format!("{idx}: {e}, at {location}").into_boxed_str());
        }

        next_error
    }

    let mut stack = Vec::with_capacity(16);

    let mut next = extract_single(&mut stack, e);

    loop {
        match next {
            NextError::Err2(e) => {
                next = extract_single(&mut stack, e);
                continue;
            }
            NextError::Std(e) => {
                let idx = stack.len();
                stack.push(format!("{idx}: {e}").into_boxed_str());
                break;
            }
            NextError::None => break,
        }
    }

    stack.into_boxed_slice()
}
