use crate::Error2;

pub fn extract_error_stack(e: &dyn Error2) -> Box<[Box<str>]> {
    let backtrace = e.backtrace();
    let (head, messages) = backtrace.head_and_messages();
    let locations = backtrace.locations();

    let mut stack: Vec<Box<str>> = Vec::with_capacity(messages.len());

    if let Some(head) = head {
        stack.push(format!("{}: {}", head.type_name(), head.display()).into());
    }

    // Iterate messages forward, using peekable() to get the next error's start_index
    let mut iter = messages.iter().peekable();

    while let Some(message) = iter.next() {
        let type_name = message.type_name();
        let display = message.display();
        let start_index = message.index();

        // Determine the location range for this error
        // From start_index to the next error's start_index (if it exists)
        let end_index = iter
            .peek()
            .map_or(locations.len(), |next_msg| next_msg.index());

        // Build error message
        let mut error_msg = format!("{}: {}", type_name, display);

        // Add location information, iterating backwards (later propagated locations shown first)
        for location in locations[start_index..end_index].iter().rev() {
            error_msg.push_str("\n    at ");
            error_msg.push_str(&location.to_string());
        }

        stack.push(error_msg.into());
    }

    // Reverse the order of error messages (later occurred errors shown first)
    stack.reverse();

    stack.into()
}

pub fn extract_error_message(e: &dyn Error2) -> Box<str> {
    let stack = extract_error_stack(e);

    if stack.is_empty() {
        Box::from("")
    } else {
        let mut buf = String::new();

        for msg in stack {
            buf.push_str(&msg);
            buf.push('\n');
        }

        buf.pop(); // Remove last newline

        buf.into()
    }
}
