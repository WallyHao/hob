// --- provider::sse ---
// Preserve bytes until a complete line arrives; transport chunks are not text boundaries.

#[derive(Default)]
pub(super) struct Events {
    line: Vec<u8>,
    data: String,
    has_data: bool,
    skip_lf: bool,
    started: bool,
}

impl Events {
    pub(super) fn push(&mut self, bytes: &[u8]) -> Result<Vec<String>, String> {
        let mut events = Vec::new();
        for byte in bytes {
            if self.skip_lf && *byte == b'\n' {
                self.skip_lf = false;
                continue;
            }
            self.skip_lf = *byte == b'\r';
            if matches!(*byte, b'\r' | b'\n') {
                self.line(&mut events)?;
            } else {
                self.line.push(*byte);
            }
        }
        Ok(events)
    }

    fn line(&mut self, events: &mut Vec<String>) -> Result<(), String> {
        let text = std::str::from_utf8(&self.line)
            .map_err(|_| "invalid UTF-8 in event stream".to_owned())?;
        let text = if self.started {
            text
        } else {
            text.trim_start_matches('\u{feff}')
        };
        self.started = true;
        if text.is_empty() {
            if self.has_data {
                self.data.pop();
                events.push(std::mem::take(&mut self.data));
                self.has_data = false;
            }
        } else {
            let (field, value) = text.split_once(':').unwrap_or((text, ""));
            if field == "data" {
                self.data.push_str(value.strip_prefix(' ').unwrap_or(value));
                self.data.push('\n');
                self.has_data = true;
            }
        }
        self.line.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Events;

    #[test]
    fn every_byte_boundary_preserves_unicode_and_events() {
        let body = "\u{feff}: comment\r\ndata: 你好 🦀\r\ndata: second\r\n\r\ndata: last\n\n";
        for split in 0..=body.len() {
            let mut parser = Events::default();
            let mut events = parser.push(&body.as_bytes()[..split]).unwrap();
            events.extend(parser.push(&body.as_bytes()[split..]).unwrap());
            assert_eq!(events, ["你好 🦀\nsecond", "last"], "split {split}");
        }
    }

    #[test]
    fn one_byte_chunks_and_cr_only_lines_work() {
        let mut parser = Events::default();
        let events: Vec<_> = "data: 你好\r\r"
            .bytes()
            .flat_map(|byte| parser.push(&[byte]).unwrap())
            .collect();
        assert_eq!(events, ["你好"]);
    }

    #[test]
    fn invalid_utf8_is_an_error_and_unfinished_events_are_not_emitted() {
        assert!(Events::default().push(b"data: \xff\n").is_err());
        assert!(
            Events::default()
                .push(b"data: unfinished\n")
                .unwrap()
                .is_empty()
        );
    }
}
