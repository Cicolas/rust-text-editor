pub fn is_crlf(c: char) -> bool {
    return c == '\n' || c == '\r';
}

pub trait TruncAt {
    fn trucate_at(&self, size: usize) -> Option<String>;
}

impl TruncAt for str {
    fn trucate_at(&self, size: usize) -> Option<String> {
        if size > self.len() {
            None
        } else {
            Some(self.split_at(size).1.to_owned())
        }
    }
}
