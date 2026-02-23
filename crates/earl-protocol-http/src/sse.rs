/// A parsed Server-Sent Event.
#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event_type: Option<String>,
    pub data: String,
    pub id: Option<String>,
}

/// Parse raw SSE text into individual events.
///
/// Events are separated by blank lines (`\n\n`).
///
/// Within each event block:
/// - Lines starting with `data:` contribute to the event data (joined with `\n`).
/// - Lines starting with `event:` set the event type.
/// - Lines starting with `id:` set the event id.
/// - Lines starting with `:` are comments and are skipped.
/// - Both `field: value` and `field:value` forms are accepted (optional space
///   after the colon).
pub fn parse_sse_events(input: &str) -> Vec<SseEvent> {
    let mut events = Vec::new();

    // Split on blank lines (the event boundary in the SSE spec).
    // A blank line is produced by two consecutive newlines.
    for block in input.split("\n\n") {
        let block = block.trim_start_matches('\n');
        if block.is_empty() {
            continue;
        }

        let mut data_lines: Vec<&str> = Vec::new();
        let mut event_type: Option<String> = None;
        let mut id: Option<String> = None;

        for line in block.lines() {
            if line.starts_with(':') {
                // Comment — skip.
                continue;
            }

            if let Some(rest) = line.strip_prefix("data:") {
                let value = rest.strip_prefix(' ').unwrap_or(rest);
                data_lines.push(value);
            } else if let Some(rest) = line.strip_prefix("event:") {
                let value = rest.strip_prefix(' ').unwrap_or(rest);
                event_type = Some(value.to_string());
            } else if let Some(rest) = line.strip_prefix("id:") {
                let value = rest.strip_prefix(' ').unwrap_or(rest);
                id = Some(value.to_string());
            }
            // Unknown fields are ignored per the SSE spec.
        }

        // Only emit an event if there was at least one data line.
        if !data_lines.is_empty() {
            events.push(SseEvent {
                event_type,
                data: data_lines.join("\n"),
                id,
            });
        }
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_data_event() {
        let input = "data: hello world\n\n";
        let events = parse_sse_events(input);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello world");
    }

    #[test]
    fn parses_multiline_data_event() {
        let input = "data: line1\ndata: line2\n\n";
        let events = parse_sse_events(input);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "line1\nline2");
    }

    #[test]
    fn parses_event_with_type() {
        let input = "event: update\ndata: {\"key\":\"value\"}\n\n";
        let events = parse_sse_events(input);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type.as_deref(), Some("update"));
        assert_eq!(events[0].data, "{\"key\":\"value\"}");
    }

    #[test]
    fn skips_comments() {
        let input = ": this is a comment\ndata: actual data\n\n";
        let events = parse_sse_events(input);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "actual data");
    }

    #[test]
    fn handles_multiple_events() {
        let input = "data: event1\n\ndata: event2\n\n";
        let events = parse_sse_events(input);
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn parses_event_with_id() {
        let input = "id: 42\ndata: payload\n\n";
        let events = parse_sse_events(input);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id.as_deref(), Some("42"));
        assert_eq!(events[0].data, "payload");
    }

    #[test]
    fn handles_no_space_after_colon() {
        let input = "data:no-space\n\n";
        let events = parse_sse_events(input);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "no-space");
    }

    #[test]
    fn ignores_block_without_data() {
        let input = "event: ping\n\n";
        let events = parse_sse_events(input);
        assert!(events.is_empty());
    }

    #[test]
    fn handles_empty_input() {
        let events = parse_sse_events("");
        assert!(events.is_empty());
    }
}
