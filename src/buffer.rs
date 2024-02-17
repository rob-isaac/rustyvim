use std::collections::HashMap;

const WINDOW_MARK_OFFSET: u8 = 2 * 26;

pub struct Buffer {
    lines: Vec<String>,
    marks: HashMap<u8, Mark>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mark {
    pub row: usize,
    pub col: usize,
}

// TODO: Allow using character indexes instead of byte indexes for columns
impl Buffer {
    pub fn new() -> Self {
        return Self::from_str("");
    }
    pub fn from_str(s: &str) -> Self {
        Self {
            lines: s.split("\n").map(String::from).collect(),
            marks: HashMap::new(),
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

    pub fn insert(&mut self, row: usize, col: usize, s: &str) {
        let mut split_iter = s.split("\n");
        let first = split_iter.next().unwrap();
        if split_iter.clone().peekable().peek() == None {
            // There is only one line - insert it into the current row
            self.lines[row].insert_str(col, first);

            for (_, mark) in self.marks.iter_mut() {
                if mark.row == row && mark.col > col {
                    mark.col += first.len();
                }
            }
        } else {
            // There are multiple lines, split the row and splice in the new data
            let mut to_add: Vec<_> = split_iter.map(String::from).collect();
            let num_new_rows = to_add.len();
            let orig_last_len = to_add.last().unwrap().len();
            if col < self.lines[row].len() {
                to_add
                    .last_mut()
                    .unwrap()
                    .insert_str(0, &self.lines[row][col..]);
                self.lines[row].replace_range(col.., first);
            } else {
                self.lines[row].push_str(first);
            }
            self.lines.splice(row + 1..row + 1, to_add);

            for (_, mark) in self.marks.iter_mut() {
                if mark.row == row && mark.col > col {
                    mark.row += num_new_rows;
                    if num_new_rows > 0 {
                        mark.col -= col;
                    }
                    mark.col += orig_last_len;
                } else if mark.row > row {
                    mark.row += num_new_rows;
                }
            }
        }
    }

    pub fn remove(&mut self, row_start: usize, col_start: usize, row_end: usize, col_end: usize) {
        if row_start == row_end {
            self.lines[row_start].replace_range(col_start..col_end, "");
            for (_, mark) in self.marks.iter_mut() {
                if mark.row == row_start && mark.col > col_start {
                    if mark.col < col_end {
                        mark.col = col_start;
                    } else {
                        mark.col -= col_end - col_start;
                    }
                }
            }
            return;
        }
        self.lines[row_start].truncate(col_start);
        if row_end < self.lines.len() {
            // TODO: avoid the extra copy
            let to_push = self.lines[row_end][col_end..].to_owned();
            self.lines[row_start].push_str(&to_push);
            self.lines.drain(row_start + 1..row_end + 1);
        } else {
            self.lines.truncate(row_start + 1);
        }

        for (_, mark) in self.marks.iter_mut() {
            if mark.row == row_start {
                if mark.col > col_start {
                    mark.col = col_start;
                }
            } else if mark.row > row_start {
                if mark.row <= row_end {
                    mark.row = row_start;
                    if mark.row < row_end || mark.col < col_end {
                        mark.col = col_start;
                    } else {
                        mark.col -= col_end;
                        mark.col += col_start;
                    }
                } else {
                    mark.row -= row_end - row_start;
                }
            }
        }
    }

    pub fn set_mark(&mut self, key: u8, mark: Mark) {
        self.marks.insert(key, mark);
    }

    pub fn get_mark(&self, key: u8) -> Mark {
        if let Some(val) = self.marks.get(&key) {
            val.clone()
        } else {
            Mark { row: 0, col: 0 }
        }
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
    fn insert() {
        let mut buf = Buffer::from_str("hello world");
        buf.insert(0, 0, " ");
        buf.insert(0, buf.get_line(0).len(), " ");
        buf.insert(0, 6, "w");
        assert_eq!(buf.to_str(), " hellow world ")
    }
    #[test]
    fn insert_multiline_str() {
        let mut buf = Buffer::from_str("hello world");
        buf.insert(0, 0, "\n");
        assert_eq!(buf.get_lines(), ["", "hello world"]);
        buf.insert(1, buf.get_line(1).len(), "\n");
        assert_eq!(buf.get_lines(), ["", "hello world", ""]);
        buf.insert(1, 5, "\n---\n");
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
        assert_eq!(buf.get_lines(), ["hnother line"]);
    }
    #[test]
    fn remove_everything() {
        let mut buf = Buffer::from_str("hello world\nanother line");
        buf.remove(0, 0, 2, 0);
        assert_eq!(buf.get_lines(), [""]);
    }

    #[test]
    fn marks_on_insert() {
        let mut buf = Buffer::from_str("hello world\nanother line");

        // put marks on "h", "a", <eol>, and " "
        buf.set_mark(0, Mark { row: 0, col: 0 });
        buf.set_mark(1, Mark { row: 1, col: 0 });
        buf.set_mark(
            2,
            Mark {
                row: 0,
                col: "hello world".len(),
            },
        );
        buf.set_mark(3, Mark { row: 0, col: 6 });

        let to_insert = "very ";
        buf.insert(0, 0, to_insert);

        // unchanged
        assert_eq!(buf.get_mark(0), Mark { row: 0, col: 0 });
        // unchanged
        assert_eq!(buf.get_mark(1), Mark { row: 1, col: 0 });
        // moved by insertion length
        assert_eq!(
            buf.get_mark(2),
            Mark {
                row: 0,
                col: "hello world".len() + to_insert.len(),
            }
        );
        // moved by insertion length
        assert_eq!(
            buf.get_mark(3),
            Mark {
                row: 0,
                col: 6 + to_insert.len(),
            }
        );
    }

    #[test]
    fn marks_on_insert_multiline() {
        let mut buf = Buffer::from_str("hello world\nanother line");

        // put marks on "h", "a", <eol>, and " "
        buf.set_mark(0, Mark { row: 0, col: 0 });
        buf.set_mark(1, Mark { row: 1, col: 0 });
        buf.set_mark(
            2,
            Mark {
                row: 0,
                col: "hello world".len(),
            },
        );
        buf.set_mark(3, Mark { row: 0, col: 6 });

        let to_insert = "ot dog\nThis is j";
        buf.insert(0, 1, to_insert);

        // unchanged
        assert_eq!(buf.get_mark(0), Mark { row: 0, col: 0 });
        // moved by a row but no column offset
        assert_eq!(buf.get_mark(1), Mark { row: 2, col: 0 });
        // moved by a row and column offset
        assert_eq!(
            buf.get_mark(2),
            Mark {
                row: 1,
                col: "This is jello world".len(),
            }
        );
        // moved by a row and column offset
        assert_eq!(
            buf.get_mark(3),
            Mark {
                row: 1,
                col: 6 + "This is j".len() - "h".len(),
            }
        );
    }

    #[test]
    fn marks_on_remove() {
        let mut buf = Buffer::from_str("hello world\nanother line");

        // put marks on "h", "a", <eol>, and " "
        buf.set_mark(0, Mark { row: 0, col: 0 });
        buf.set_mark(1, Mark { row: 1, col: 0 });
        buf.set_mark(
            2,
            Mark {
                row: 0,
                col: "hello world".len(),
            },
        );
        buf.set_mark(3, Mark { row: 0, col: 6 });

        buf.remove(0, 1, 0, 1 + "ello wo".len());

        // unchanged
        assert_eq!(buf.get_mark(0), Mark { row: 0, col: 0 });
        // unchanged
        assert_eq!(buf.get_mark(1), Mark { row: 1, col: 0 });
        // moved by removal length
        assert_eq!(
            buf.get_mark(2),
            Mark {
                row: 0,
                col: "hello world".len() - "ello wo".len(),
            }
        );
        // moved to the beginning of the removal
        assert_eq!(buf.get_mark(3), Mark { row: 0, col: 1 });
    }

    #[test]
    fn marks_on_remove_multiline() {
        let mut buf = Buffer::from_str("hello world\nanother line");

        // put marks on "h", "a", <eol>, and "l"
        buf.set_mark(0, Mark { row: 0, col: 0 });
        buf.set_mark(1, Mark { row: 1, col: 0 });
        buf.set_mark(
            2,
            Mark {
                row: 0,
                col: "hello world".len(),
            },
        );
        buf.set_mark(3, Mark { row: 1, col: 8 });

        buf.remove(0, 1, 1, 2);

        // unchanged
        assert_eq!(buf.get_mark(0), Mark { row: 0, col: 0 });
        // moved to the beginning of the removal
        assert_eq!(buf.get_mark(1), Mark { row: 0, col: 1 });
        // moved to the beginning of the removal
        assert_eq!(buf.get_mark(2), Mark { row: 0, col: 1 });
        // moved by the deleted section
        assert_eq!(buf.get_mark(3), Mark { row: 0, col: 7 });
    }
}
