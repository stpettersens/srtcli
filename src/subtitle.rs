/* Part of srtcli */
use std::fmt;

pub struct Subtitle {
    seq: i64,
    start: i64,
    end: i64,
    text: String,
}

impl Subtitle {
    pub fn new(seq: i64, start: i64, end: i64, text: &str) -> Subtitle {
        Subtitle {
            seq: seq,
            start: start,
            end: end,
            text: text.to_string(),
        }
    }
    pub fn get_start(&self) -> i64 {
        self.start
    }
    pub fn get_end(&self) -> i64 {
        self.end
    }
    pub fn get_text(&self) -> &str {
        &self.text
    }
}

impl fmt::Display for Subtitle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Subtitle #{} ({} -> {}): {}",
        self.seq, self.start, self.end, self.text)
    }
}
