use anyhow::anyhow;
use anyhow::Result;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::mem;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Buffer {
    lines: Vec<String>,
    filename: Option<PathBuf>,
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer {
            lines: vec![String::new()],
            filename: None,
        }
    }
}

impl Buffer {
    pub(crate) fn new() -> Buffer {
        Default::default()
    }

    pub(crate) fn from_filename(filename: &Path) -> Result<Buffer> {
        if filename.exists() {
            if filename.is_file() {
                return Ok(Buffer {
                    lines: read_lines(File::open(&filename)?)?,
                    filename: Some(filename.to_path_buf()),
                });
            }
            return Err(anyhow!(
                "Directories and symlinks are not currently supported!"
            ));
        }
        Ok(Buffer {
            filename: Some(filename.to_path_buf()),
            ..Default::default()
        })
    }

    fn from_str(lines: &str) -> Result<Buffer> {
        Ok(Buffer {
            lines: read_lines(lines.as_bytes())?,
            filename: None,
        })
    }

    pub(crate) fn lines(&self) -> &[String] {
        return &self.lines;
    }

    pub(crate) fn filename(&self) -> Option<&Path> {
        if let Some(filename) = &self.filename {
            return Some(&filename);
        }
        None
    }

    pub(crate) fn insert(&mut self, row: usize, col: usize, val: char) {
        self.lines[row].insert(col, val);
    }

    pub(crate) fn insert_line(&mut self, row: usize) {
        self.lines.insert(row, String::new());
    }

    pub(crate) fn num_lines(&self) -> usize {
        self.lines.len()
    }

    pub(crate) fn line_len(&self, row: usize) -> usize {
        self.lines[row].len()
    }

    pub(crate) fn remove(&mut self, row: usize, col: usize) {
        self.lines[row].remove(col);
    }

    pub(crate) fn join_below(&mut self, row: usize) {
        if row < self.lines.len() - 1 {
            let tmp = mem::take(&mut self.lines[row + 1]);
            self.lines[row].push_str(&tmp);
            self.lines.remove(row);
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

    #[test]
    fn test_lines() {
        let lines = vec![String::from("Line1"), String::from("Line2")];
        assert_eq!(
            Buffer {
                lines: lines.clone(),
                ..Default::default()
            }
            .lines(),
            lines
        )
    }

    #[test]
    fn test_filename() -> Result<()> {
        let dummy_path = Path::new("/tmp/dummy.txt");
        assert_eq!(
            Buffer {
                filename: Some(dummy_path.to_path_buf()),
                ..Default::default()
            }
            .filename(),
            Some(dummy_path)
        );
        Ok(())
    }

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

    #[test]
    fn test_insert() -> Result<()> {
        let mut buf = Buffer::from_str("abcdefg\nhijklmnop")?;
        buf.insert(0, 0, 'z');
        buf.insert(1, 3, 'y');
        buf.insert(1, 10, 'x');
        assert_eq!(buf.lines(), vec!["zabcdefg", "hijyklmnopx"]);
        Ok(())
    }

    #[test]
    fn test_insert_row() -> Result<()> {
        let mut buf = Buffer::from_str("abcdefg\nhijklmnop")?;
        buf.insert_line(0);
        buf.insert_line(3);
        buf.insert_line(3);
        assert_eq!(buf.lines(), vec!["", "abcdefg", "hijklmnop", "", ""]);
        Ok(())
    }
}
