/// A parsed Server-Sent Event.
#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event_type: Option<String>,
    pub data: String,
    pub id: Option<String>,
}

/// Stateful Server-Sent Events parser.
///
/// Buffers incomplete events across calls to [`feed`] so that events split
/// across HTTP chunk boundaries are handled correctly.
pub struct SseParser {
    /// Leftover text that hasn't yet formed a complete event block.
    buffer: String,
}

impl Default for SseParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SseParser {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// Feed a chunk of text into the parser, returning any complete events.
    ///
    /// Incomplete events are buffered internally and will be emitted on
    /// subsequent `feed()` calls once the closing blank line arrives.
    pub fn feed(&mut self, input: &str) -> Vec<SseEvent> {
        self.buffer.push_str(input);
        let mut events = Vec::new();

        // The SSE spec defines event boundaries as blank lines.
        // We support both \n\n and \r\n\r\n.
        loop {
            // Find the next event boundary (blank line).
            let boundary = self
                .buffer
                .find("\n\n")
                .map(|pos| (pos, 2))
                .or_else(|| self.buffer.find("\r\n\r\n").map(|pos| (pos, 4)));

            let Some((pos, sep_len)) = boundary else {
                break;
            };

            let block = &self.buffer[..pos];
            if let Some(event) = Self::parse_block(block) {
                events.push(event);
            }
            // Drain the consumed block + separator.
            self.buffer.drain(..pos + sep_len);
        }

        events
    }

    /// Flush any remaining buffered data as a final event.
    ///
    /// Call this when the stream ends to emit any trailing event that
    /// wasn't followed by a blank line.
    pub fn flush(&mut self) -> Option<SseEvent> {
        let block = std::mem::take(&mut self.buffer);
        let block = block.trim();
        if block.is_empty() {
            return None;
        }
        Self::parse_block(block)
    }

    fn parse_block(block: &str) -> Option<SseEvent> {
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
        if data_lines.is_empty() {
            return None;
        }

        Some(SseEvent {
            event_type,
            data: data_lines.join("\n"),
            id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_data_event() {
        let input = "data: hello world\n\n";
        let events = SseParser::new().feed(input);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello world");
    }

    #[test]
    fn parses_multiline_data_event() {
        let input = "data: line1\ndata: line2\n\n";
        let events = SseParser::new().feed(input);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "line1\nline2");
    }

    #[test]
    fn parses_event_with_type() {
        let input = "event: update\ndata: {\"key\":\"value\"}\n\n";
        let events = SseParser::new().feed(input);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type.as_deref(), Some("update"));
        assert_eq!(events[0].data, "{\"key\":\"value\"}");
    }

    #[test]
    fn skips_comments() {
        let input = ": this is a comment\ndata: actual data\n\n";
        let events = SseParser::new().feed(input);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "actual data");
    }

    #[test]
    fn handles_multiple_events() {
        let input = "data: event1\n\ndata: event2\n\n";
        let events = SseParser::new().feed(input);
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn parses_event_with_id() {
        let input = "id: 42\ndata: payload\n\n";
        let events = SseParser::new().feed(input);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id.as_deref(), Some("42"));
        assert_eq!(events[0].data, "payload");
    }

    #[test]
    fn handles_no_space_after_colon() {
        let input = "data:no-space\n\n";
        let events = SseParser::new().feed(input);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "no-space");
    }

    #[test]
    fn ignores_block_without_data() {
        let input = "event: ping\n\n";
        let events = SseParser::new().feed(input);
        assert!(events.is_empty());
    }

    #[test]
    fn handles_empty_input() {
        let events = SseParser::new().feed("");
        assert!(events.is_empty());
    }

    #[test]
    fn event_split_across_chunks() {
        let mut parser = SseParser::new();

        // First chunk contains the beginning of the event but no blank-line terminator.
        let events = parser.feed("data: hel");
        assert!(events.is_empty(), "no complete event yet");

        // Second chunk completes the event with the rest of the data + blank line.
        let events = parser.feed("lo world\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello world");
    }

    #[test]
    fn handles_crlf_line_endings() {
        let input = "event: update\r\ndata: payload\r\n\r\n";
        let events = SseParser::new().feed(input);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type.as_deref(), Some("update"));
        assert_eq!(events[0].data, "payload");
    }

    #[test]
    fn flush_trailing_event() {
        let mut parser = SseParser::new();

        // Feed an event that is NOT terminated by a blank line.
        let events = parser.feed("data: trailing");
        assert!(events.is_empty(), "no blank line yet, so nothing emitted");

        // Flush should emit the trailing event.
        let event = parser.flush().expect("should flush trailing event");
        assert_eq!(event.data, "trailing");
    }

    #[test]
    fn multiple_feed_calls() {
        let mut parser = SseParser::new();

        // First feed: one complete event and start of another.
        let events = parser.feed("data: first\n\ndata: sec");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "first");

        // Second feed: finish the second event and deliver a third.
        let events = parser.feed("ond\n\ndata: third\n\n");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].data, "second");
        assert_eq!(events[1].data, "third");
    }
}
