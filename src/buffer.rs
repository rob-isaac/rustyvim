use std::collections::HashMap;

// NOTE: We prob want to move the bufferlist module into here at some point
// NOTE: We prob want an InsertBuffer type which buffers an insert-mode insertion
// so that we don't have O(N^2) perf
// TODO: Need to test/implement proper behavior for out-of-bounds marks. E.g. if we are inserting
// in an out-of-bounds area, do we want to pad with whitespace to make it in-bounds or to clamp the
// insertion begin area to be in-bounds? Additionally, when moving downwards through lines of
// different length, we want our mark to display like the following:
// aaaaaaaaaa[a]aaaaaaa
// bbb[b]
// dddddddddd[d]d
//
// But if we stop to insert in the b-line we probably want the following:
// aaaaaaaaaa[a]aaaaaaa
// bbb[b]
// ddd[d]dddddddd
//
// This leads to the heuristic of allowing OOB until insertion, at which point the mark gets
// clamped.
// TODO: Allow using character indexes instead of byte indexes for columns
// TODO: Folds

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

impl Mark {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
    fn move_member(member: &mut usize, delta: i64) {
        if delta < 0 {
            let delta = (-delta) as usize;
            *member = if delta >= *member { 0 } else { *member - delta }
        } else {
            *member += delta as usize;
        }
    }
    pub fn move_row(&mut self, delta: i64) {
        Self::move_member(&mut self.row, delta)
    }
    pub fn move_col(&mut self, delta: i64) {
        Self::move_member(&mut self.col, delta)
    }
    pub fn clamp_row(&mut self, max: usize) {
        if self.row > max {
            self.row = max;
        }
    }
    pub fn clamp_col(&mut self, max: usize) {
        if self.col > max {
            self.col = max;
        }
    }
}

impl Buffer {
    pub fn new() -> Self {
        Self::from_str("")
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

    pub fn insert(&mut self, insert_mark: Mark, s: &str) {
        let (row, col) = (insert_mark.row, insert_mark.col);
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

    pub fn remove(&mut self, mark_start: Mark, mark_end: Mark) {
        let (row_start, col_start, row_end, col_end) =
            (mark_start.row, mark_start.col, mark_end.row, mark_end.col);
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
                    if mark.row < row_end || mark.col < col_end {
                        mark.col = col_start;
                    } else {
                        mark.col -= col_end;
                        mark.col += col_start;
                    }
                    mark.row = row_start;
                } else {
                    mark.row -= row_end - row_start;
                }
            }
        }
    }

    pub fn set_mark(&mut self, key: u8, mark: Mark) {
        self.marks.insert(key, mark);
    }

    pub fn move_mark(&mut self, key: u8, drow: i64, dcol: i64) {
        if let Some(mark) = self.marks.get_mut(&key) {
            mark.move_row(drow);
            mark.move_col(dcol);
        } else {
            let drow = if drow < 0 { 0 } else { drow };
            let dcol = if dcol < 0 { 0 } else { dcol };
            self.marks
                .insert(key, Mark::new(drow as usize, dcol as usize));
        }
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
        buf.insert(Mark::new(0, 0), " ");
        buf.insert(Mark::new(0, buf.get_line(0).len()), " ");
        buf.insert(Mark::new(0, 6), "w");
        assert_eq!(buf.to_str(), " hellow world ")
    }
    #[test]
    fn insert_multiline_str() {
        let mut buf = Buffer::from_str("hello world");
        buf.insert(Mark::new(0, 0), "\n");
        assert_eq!(buf.get_lines(), ["", "hello world"]);
        buf.insert(Mark::new(1, buf.get_line(1).len()), "\n");
        assert_eq!(buf.get_lines(), ["", "hello world", ""]);
        buf.insert(Mark::new(1, 5), "\n---\n");
        assert_eq!(buf.get_lines(), ["", "hello", "---", " world", ""])
    }
    #[test]
    fn remove() {
        let mut buf = Buffer::from_str("hello world");
        buf.remove(Mark::new(0, 0), Mark::new(0, 5));
        assert_eq!(buf.get_lines(), [" world"]);
    }
    #[test]
    fn remove_multiline() {
        let mut buf = Buffer::from_str("hello world\nanother line");
        buf.remove(Mark::new(0, 1), Mark::new(1, 1));
        assert_eq!(buf.get_lines(), ["hnother line"]);
    }
    #[test]
    fn remove_everything() {
        let mut buf = Buffer::from_str("hello world\nanother line");
        buf.remove(Mark::new(0, 0), Mark::new(2, 0));
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
        buf.insert(Mark::new(0, 0), to_insert);

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
        buf.insert(Mark::new(0, 1), to_insert);

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

        buf.remove(Mark::new(0, 1), Mark::new(0, 1 + "ello wo".len()));

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

        buf.remove(Mark::new(0, 1), Mark::new(1, 2));

        // unchanged
        assert_eq!(buf.get_mark(0), Mark { row: 0, col: 0 });
        // moved to the beginning of the removal
        assert_eq!(buf.get_mark(1), Mark { row: 0, col: 1 });
        // moved to the beginning of the removal
        assert_eq!(buf.get_mark(2), Mark { row: 0, col: 1 });
        // moved by the deleted section
        assert_eq!(buf.get_mark(3), Mark { row: 0, col: 7 });
    }

    #[test]
    fn mark_methods() {
        let mut m = Mark::new(0, 0);

        m.move_col(-1);
        m.move_row(-1);
        assert_eq!(m, Mark::new(0, 0));

        m.move_col(1);
        m.move_row(1);
        assert_eq!(m, Mark::new(1, 1));

        m.clamp_row(0);
        m.clamp_col(0);
        assert_eq!(m, Mark::new(0, 0));
    }
}
