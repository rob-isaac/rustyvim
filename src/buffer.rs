use anyhow::anyhow;
use anyhow::Result;
use std::cmp::min;
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
    cursors: Vec<CursorPos>,
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer {
            lines: vec![String::new()],
            filename: None,
            cursors: vec![CursorPos {
                ..Default::default()
            }],
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
        self.cursors.get(id.0).cloned()
    }

    fn get_cursor_mut(&mut self, id: WindowId) -> &mut CursorPos {
        let id_raw = id.0;
        if id_raw >= self.cursors.len() {
            self.cursors.resize_with(id_raw + 1, Default::default);
        }
        &mut self.cursors[id_raw]
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
        for cursor in &mut self.cursors {
            cursor.col += (cursor.row == row && cursor.col >= col) as usize
        }
    }

    pub(crate) fn insert_line(&mut self, row: usize) {
        self.lines.insert(row, String::new());
        for cursor in &mut self.cursors {
            cursor.row += (cursor.row >= row) as usize
        }
    }

    pub(crate) fn remove(&mut self, row: usize, col: usize) {
        self.lines[row].remove(col);
    }

    pub(crate) fn remove_line(&mut self, row: usize) {
        self.lines.remove(row);
    }

    pub(crate) fn join_above(&mut self, row: usize) {
        if row != 0 {
            let tmp = mem::take(&mut self.lines[row]);
            self.lines[row - 1].push_str(&tmp);
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

    /// A test fixture to create a buffer from a string
    fn buf_from_str(s: &str) -> Result<Buffer> {
        let mut cursors = Vec::new();
        let mut lines = Vec::new();
        for (row, line) in s.lines().enumerate() {
            let mut col = 0;
            let mut chunks = Vec::new();
            let mut matches = 0;
            while let Some(relative_col) = line[col..].find("|") {
                chunks.push(&line[col..col + relative_col]);
                cursors.push(CursorPos {
                    row,
                    col: col + relative_col - matches,
                });
                col += relative_col + 1;
                matches += 1;
            }
            chunks.push(&line[col..]);
            lines.push(chunks.join(""));
        }
        if cursors.is_empty() {
            panic!("couldn't find cursor position! Indicated with the '|' symbol")
        }
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
        let mut buf = buf_from_str("|abcdefg\n|hij|klmnop|")?;
        assert_eq!(
            buf.cursors,
            vec![
                CursorPos { row: 0, col: 0 },
                CursorPos { row: 1, col: 0 },
                CursorPos { row: 1, col: 3 },
                CursorPos { row: 1, col: 9 },
            ]
        );
        buf.insert(0, 0, 'z');
        buf.insert(1, 3, 'y');
        buf.insert(1, 10, 'x');
        buf.insert(1, 10, 'w');
        assert_eq!(buf.lines, vec!["zabcdefg", "hijyklmnopwx"]);
        assert_eq!(
            buf.cursors,
            vec![
                CursorPos { row: 0, col: 1 },
                CursorPos { row: 1, col: 0 },
                CursorPos { row: 1, col: 4 },
                CursorPos { row: 1, col: 12 },
            ]
        );
        Ok(())
    }

    /// Tests the insert_row method
    #[test]
    fn test_insert_row() -> Result<()> {
        let mut buf = buf_from_str("a|bcdefg\n|hijklmnop|")?;
        assert_eq!(
            buf.cursors,
            vec![
                CursorPos { row: 0, col: 1 },
                CursorPos { row: 1, col: 0 },
                CursorPos { row: 1, col: 9 },
            ]
        );
        buf.insert_line(0);
        buf.insert_line(2);
        buf.insert_line(4);
        assert_eq!(buf.lines(), vec!["", "abcdefg", "", "hijklmnop", ""]);
        assert_eq!(
            buf.cursors,
            vec![
                CursorPos { row: 1, col: 1 },
                CursorPos { row: 3, col: 0 },
                CursorPos { row: 3, col: 9 },
            ]
        );
        Ok(())
    }
}
