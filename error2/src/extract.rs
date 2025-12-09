use crate::{BakctraceEntry, Error2, Location};

pub fn extract_error_stack(e: &dyn Error2) -> Box<[Box<str>]> {
    let backtrace = e.backtrace();
    let (head, entries) = backtrace.head_and_entries();

    let mut stack: Vec<Box<str>> = Vec::with_capacity(entries.len());

    if let Some(head) = head {
        stack.push(format!("{}: {}", head.type_name(), head.display()).into());
    }

    {
        let mut msg = String::new();
        let mut locations = Vec::<&Location>::new();

        fn flush_msg(mut msg: String, locations: &mut Vec<&Location>, stack: &mut Vec<Box<str>>) {
            if msg.is_empty() {
                debug_assert!(locations.is_empty());
                return;
            }

            for location in locations.iter() {
                msg.push_str("\n    at ");
                msg.push_str(&location.to_string());
            }

            stack.push(msg.into());
            locations.clear();
        }

        for entry in entries {
            match entry {
                BakctraceEntry::Message(message) => {
                    flush_msg(msg, &mut locations, &mut stack);

                    msg = format!("{}: {}", message.type_name(), message.display());
                }
                BakctraceEntry::Locations(dl) => {
                    let [first, second] = dl.inner();

                    debug_assert!(!first.is_uninit());
                    locations.push(first);

                    if !second.is_uninit() {
                        locations.push(second);
                    }
                }
            }
        }

        flush_msg(msg, &mut locations, &mut stack);
    }

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
