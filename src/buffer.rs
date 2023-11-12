use anyhow::anyhow;
use anyhow::Result;
use std::cmp::min;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::mem;
use std::path::Path;
use std::path::PathBuf;

use crate::window::WindowId;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub(crate) struct CursorPos {
    pub(crate) row: usize,
    pub(crate) col: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Buffer {
    lines: Vec<String>,
    filename: Option<PathBuf>,
    cursors: HashMap<WindowId, CursorPos>,
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer {
            lines: vec![String::new()],
            ..Default::default()
        }
    }
}

impl Buffer {
    pub(crate) fn new() -> Buffer {
        Default::default()
    }

    // pub(crate) fn from_filename(filename: &Path) -> Result<Buffer> {
    //     if filename.exists() {
    //         if filename.is_file() {
    //             return Ok(Buffer {
    //                 lines: read_lines(File::open(&filename)?)?,
    //                 filename: Some(filename.to_path_buf()),
    //             });
    //         }
    //         return Err(anyhow!(
    //             "Directories and symlinks are not currently supported!"
    //         ));
    //     }
    //     Ok(Buffer {
    //         filename: Some(filename.to_path_buf()),
    //         ..Default::default()
    //     })
    // }

    pub(crate) fn lines(&self) -> &[String] {
        return &self.lines;
    }

    pub(crate) fn filename(&self) -> Option<&Path> {
        if let Some(filename) = &self.filename {
            return Some(&filename);
        }
        None
    }

    pub(crate) fn get_cursor(&self, id: WindowId) -> Option<CursorPos> {
        self.cursors.get(&id).cloned()
    }

    fn get_cursor_mut(&mut self, id: WindowId) -> &mut CursorPos {
        self.cursors.entry(id).or_default()
    }

    pub(crate) fn set_cursor(&mut self, id: WindowId, cpos: CursorPos) {
        *self.get_cursor_mut(id) = cpos;
    }

    pub(crate) fn move_cursor(&mut self, id: WindowId, rows: i32, cols: i32) {
        let mut cpos = self.get_cursor_mut(id).clone();
        if rows < 0 {
            let rows = (-rows) as usize;
            if rows > cpos.row {
                cpos.row = 0;
            } else {
                cpos.row -= rows;
            }
        } else {
            let rows = rows as usize;
            cpos.row = min(cpos.row + rows, self.lines.len());
        }

        // don't clamp unless actually moving columns
        if cols != 0 {
            if cols < 0 {
                let cols = (-cols) as usize;
                if cols > cpos.col {
                    cpos.col = 0;
                } else {
                    cpos.col -= cols;
                }
            } else {
                let cols = cols as usize;
                cpos.col = min(
                    cpos.col + cols,
                    self.lines.get(cpos.row).unwrap_or(&"".to_string()).len(),
                );
            }
        }
        self.set_cursor(id, cpos);
    }

    pub(crate) fn insert(&mut self, row: usize, col: usize, val: char) {
        self.lines[row].insert(col, val);
        for (_, cursor) in &mut self.cursors {
            cursor.col += (cursor.row == row && cursor.col >= col) as usize
        }
    }

    pub(crate) fn insert_line(&mut self, row: usize) {
        self.lines.insert(row, String::new());
        for (_, cursor) in &mut self.cursors {
            cursor.row += (cursor.row >= row) as usize
        }
    }

    pub(crate) fn remove(&mut self, row: usize, col: usize) {
        self.lines[row].remove(col);
        for (_, cursor) in &mut self.cursors {
            cursor.col -= (cursor.row == row && cursor.col > col) as usize
        }
    }

    pub(crate) fn remove_line(&mut self, row: usize) {
        self.lines.remove(row);
        for (_, cursor) in &mut self.cursors {
            cursor.row -= (cursor.row != 0 && cursor.row >= row) as usize
        }
    }

    pub(crate) fn join_above(&mut self, row: usize) {
        if row != 0 {
            let tmp = mem::take(&mut self.lines[row]);
            let above_line_len = self.lines[row - 1].len();
            self.lines[row - 1].push_str(&tmp);
            for (_, cursor) in &mut self.cursors {
                if cursor.row == row {
                    cursor.col += above_line_len;
                }
            }
            // handles the row adjustments
            self.remove_line(row);
        }
    }
}

fn read_lines(read: impl io::Read) -> Result<Vec<String>> {
    let mut res: Vec<String> = BufReader::new(read)
        .lines()
        .collect::<std::io::Result<_>>()?;
    if res.is_empty() {
        res.push(String::new());
    }
    return Ok(res);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A test fixture to create a buffer from a string.
    /// Digits 0-9 are used to indicate cursor positions rather than text.
    fn buf_from_lines(input_lines: &[&str]) -> Result<Buffer> {
        let mut cursors = HashMap::new();
        let lines: Vec<String> = input_lines
            .iter()
            .enumerate()
            .map(|(row, line)| {
                let mut line_cursors = 0;
                line.char_indices()
                    .filter_map(|(col, c)| {
                        if let Some(d) = c.to_digit(10) {
                            if cursors
                                .insert(
                                    WindowId(d as usize),
                                    CursorPos {
                                        row,
                                        col: col - line_cursors,
                                    },
                                )
                                .is_some()
                            {
                                panic!("duplicate key {}", d)
                            }
                            line_cursors += 1;
                            return None;
                        }
                        Some(c)
                    })
                    .collect()
            })
            .collect();

        Ok(Buffer {
            lines,
            filename: None,
            cursors,
        })
    }

    /// Tests the read_lines helper function
    #[test]
    fn test_read_lines() -> Result<()> {
        let empty = "";
        let one_line = "A single line";
        let two_lines = "First line.\nSecond line.";
        let carriage_return = "First line.\r\nSecond line.";
        assert_eq!(read_lines(empty.as_bytes())?, vec![String::new()]);
        assert_eq!(read_lines(one_line.as_bytes())?, vec![one_line.to_string()]);
        assert_eq!(
            read_lines(two_lines.as_bytes())?,
            two_lines.lines().map(String::from).collect::<Vec<String>>()
        );
        assert_eq!(
            read_lines(carriage_return.as_bytes())?,
            carriage_return
                .lines()
                .map(String::from)
                .collect::<Vec<String>>()
        );
        Ok(())
    }

    /// Tests the insert method
    #[test]
    fn test_insert() -> Result<()> {
        let mut buf = buf_from_lines(&["0abcdefg", "1hij2klmnop3"])?;
        buf.insert(0, 0, 'z');
        assert_eq!(buf, buf_from_lines(&["z0abcdefg", "1hij2klmnop3"])?);
        buf.insert(1, 3, 'y');
        assert_eq!(buf, buf_from_lines(&["z0abcdefg", "1hijy2klmnop3"])?);
        buf.insert(1, 10, 'x');
        assert_eq!(buf, buf_from_lines(&["z0abcdefg", "1hijy2klmnopx3"])?);
        buf.insert(1, 10, 'w');
        assert_eq!(buf, buf_from_lines(&["z0abcdefg", "1hijy2klmnopwx3"])?);
        Ok(())
    }

    /// Tests the insert_row method
    #[test]
    fn test_insert_row() -> Result<()> {
        let mut buf = buf_from_lines(&["a0bcdefg", "1hijklmnop2"])?;
        buf.insert_line(0);
        assert_eq!(buf, buf_from_lines(&["", "a0bcdefg", "1hijklmnop2"])?);
        buf.insert_line(2);
        assert_eq!(buf, buf_from_lines(&["", "a0bcdefg", "", "1hijklmnop2"])?);
        buf.insert_line(4);
        assert_eq!(
            buf,
            buf_from_lines(&["", "a0bcdefg", "", "1hijklmnop2", ""])?
        );
        Ok(())
    }

    /// Tests the remove method
    #[test]
    fn test_remove() -> Result<()> {
        let mut buf = buf_from_lines(&["a0bc", "1def2"])?;
        buf.remove(0, 1);
        assert_eq!(buf, buf_from_lines(&["a0c", "1def2"])?);
        buf.remove(0, 0);
        assert_eq!(buf, buf_from_lines(&["0c", "1def2"])?);
        buf.remove(1, 0);
        assert_eq!(buf, buf_from_lines(&["0c", "1ef2"])?);
        buf.remove(1, 1);
        assert_eq!(buf, buf_from_lines(&["0c", "1e2"])?);
        Ok(())
    }

    /// Tests the remove_line method
    #[test]
    fn test_remove_line() -> Result<()> {
        let mut buf = buf_from_lines(&["a0bc", "de", "1fg2"])?;
        buf.remove_line(0);
        assert_eq!(buf, buf_from_lines(&["d0e", "1fg2"])?);
        buf.remove_line(1);
        assert_eq!(buf, buf_from_lines(&["1d0e2"])?);
        // TODO: We should test what happens when we remove a line and the cursor is now
        // off the end. We don't have a good way to represent this in the buf_from_lines paradigm
        Ok(())
    }

    /// Tests the join_above method
    #[test]
    fn test_join_above() -> Result<()> {
        let mut buf = buf_from_lines(&["a0bc", "de", "1fg2"])?;
        buf.join_above(0);
        assert_eq!(buf, buf_from_lines(&["a0bc", "de", "1fg2"])?);
        buf.join_above(1);
        assert_eq!(buf, buf_from_lines(&["a0bcde", "1fg2"])?);
        buf.join_above(1);
        assert_eq!(buf, buf_from_lines(&["a0bcde1fg2"])?);
        Ok(())
    }
}
