pub struct Buffer {
    lines: Vec<String>,
}

impl Buffer {
    pub fn new() -> Self {
        return Self::from_str("");
    }
    pub fn from_str(s: &str) -> Self {
        Self {
            lines: s.split("\n").map(String::from).collect(),
        }
    }
    pub fn to_str(&self) -> String {
        self.lines.join("\n")
    }
    pub fn get_line(&self, i: usize) -> &str {
        &self.lines[i]
    }
    pub fn get_lines(&self) -> &[String] {
        &self.lines
    }
    // FIXME: The mutation functions in this class use byte-ranges rather than character ranges
    pub fn insert_str(&mut self, row: usize, col: usize, s: &str) {
        let mut split = s.split("\n");
        let first = split.next().unwrap();
        if split.clone().peekable().peek() == None {
            self.lines[row].insert_str(col, first);
        } else {
            let mut to_add: Vec<_> = split.map(String::from).collect();
            if col < self.lines[row].len() {
                to_add
                    .last_mut()
                    .unwrap()
                    .insert_str(0, &self.lines[row][col..]);
                self.lines[row].replace_range(col.., first);
            }
            self.lines.splice(row + 1..row + 1, to_add);
        }
    }
    pub fn remove(&mut self, row_start: usize, col_start: usize, row_end: usize, col_end: usize) {
        if row_start == row_end {
            self.lines[row_start].replace_range(col_start..col_end, "");
            return;
        }
        self.lines[row_start].truncate(col_start);
        if row_end < self.lines.len() {
            self.lines[row_end].replace_range(..col_end, "");
        }
        self.lines.drain(row_start + 1..row_end);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_empty() {
        assert_eq!(Buffer::new().to_str(), "");
    }
    #[test]
    fn create_from_string() {
        let str = "hello world";
        assert_eq!(Buffer::from_str(str).to_str(), str);
    }
    #[test]
    fn create_from_multiline_string() {
        let str = "hello world\nanother line";
        assert_eq!(Buffer::from_str(str).to_str(), str);
    }
    #[test]
    fn get_lines() {
        assert_eq!(
            Buffer::from_str("hello world\nanother line").get_lines(),
            ["hello world", "another line"]
        );
    }
    #[test]
    fn get_line() {
        assert_eq!(
            Buffer::from_str("hello world\nanother line").get_line(0),
            "hello world"
        );
    }
    #[test]
    fn insert_str() {
        let mut buf = Buffer::from_str("hello world");
        buf.insert_str(0, 0, " ");
        buf.insert_str(0, buf.get_line(0).len(), " ");
        buf.insert_str(0, 6, "w");
        assert_eq!(buf.to_str(), " hellow world ")
    }
    #[test]
    fn insert_multiline_str() {
        let mut buf = Buffer::from_str("hello world");
        buf.insert_str(0, 0, "\n");
        assert_eq!(buf.get_lines(), ["", "hello world"]);
        buf.insert_str(1, buf.get_line(1).len(), "\n");
        assert_eq!(buf.get_lines(), ["", "hello world", ""]);
        buf.insert_str(1, 5, "\n---\n");
        assert_eq!(buf.get_lines(), ["", "hello", "---", " world", ""])
    }
    #[test]
    fn remove() {
        let mut buf = Buffer::from_str("hello world");
        buf.remove(0, 0, 0, 5);
        assert_eq!(buf.get_lines(), [" world"]);
    }
    #[test]
    fn remove_multiline() {
        let mut buf = Buffer::from_str("hello world\nanother line");
        buf.remove(0, 1, 1, 1);
        assert_eq!(buf.get_lines(), ["h", "nother line"]);
    }
    #[test]
    fn remove_everything() {
        let mut buf = Buffer::from_str("hello world\nanother line");
        buf.remove(0, 0, 2, 0);
        assert_eq!(buf.get_lines(), [""]);
    }
}
