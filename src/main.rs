use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::cmp::{max, min};
use std::fs::{self, File};
use std::io::{stderr, BufRead, BufReader, Cursor, Write};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use anyhow::{anyhow, Result};
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, ModifierKeyCode,
};
use crossterm::style::{Print, Stylize};
use crossterm::terminal::{self, disable_raw_mode, enable_raw_mode, Clear, EnterAlternateScreen};
use crossterm::{cursor, execute, queue, style};

const REFRESH_RATE_MILIS: u64 = 20;

struct CursorPos {
    row: u16,
    col: u16,
}

struct Buffer {
    lines: Vec<String>,
    filename: Option<PathBuf>,
}

struct Window {
    buf: Rc<RefCell<Buffer>>,
    cpos: CursorPos,
}

struct Tab {
    windows: Vec<Window>,
    cur_win: usize,
}

enum Mode {
    Normal,
    Insert,
}

struct App {
    buffers: Vec<Rc<RefCell<Buffer>>>,
    tabs: Vec<Tab>,
    cur_tab: usize,
    mode: Mode,
}

impl App {
    fn setup() -> Result<()> {
        execute!(stderr(), EnterAlternateScreen)?;
        enable_raw_mode()?;
        queue!(stderr(), cursor::MoveTo(0, 0), cursor::Show)?;
        Ok(())
    }

    fn teardown() -> Result<()> {
        let r1 = disable_raw_mode();
        let r2 = execute!(
            stderr(),
            Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        );
        r1?;
        r2?;
        Ok(())
    }

    fn new() -> Result<App> {
        Self::setup()?;
        let default_buf = Rc::new(RefCell::new(Buffer {
            lines: vec![String::new()],
            filename: None,
        }));
        Ok(App {
            buffers: vec![default_buf.clone()],
            tabs: vec![Tab {
                windows: vec![Window {
                    buf: default_buf,
                    cpos: CursorPos { row: 0, col: 0 },
                }],
                cur_win: 0,
            }],
            cur_tab: 0,
            mode: Mode::Normal,
        })
    }

    fn from_files(files: &[&str]) -> Result<App> {
        Self::setup()?;
        let bufs = files
            .iter()
            .map(|s| -> Result<Rc<RefCell<Buffer>>> {
                let fname = PathBuf::from(s);
                if fname.exists() {
                    if fname.is_file() {
                        return Ok(Rc::new(RefCell::new(Buffer {
                            lines: BufReader::new(File::open(&fname)?)
                                .lines()
                                .collect::<std::io::Result<_>>()?,
                            filename: Some(fname),
                        })));
                    }
                    return Err(anyhow!(
                        "Directories and symlinks are not currently supported!"
                    ));
                }
                Ok(Rc::new(RefCell::new(Buffer {
                    lines: vec![String::new()],
                    filename: Some(fname),
                })))
            })
            .collect::<Result<Vec<_>>>()?;

        if let Some(last_buf) = bufs.last().cloned() {
            return Ok(App {
                buffers: bufs,
                tabs: vec![Tab {
                    windows: vec![Window {
                        buf: last_buf.clone(),
                        cpos: CursorPos { row: 0, col: 0 },
                    }],
                    cur_win: 0,
                }],
                cur_tab: 0,
                mode: Mode::Normal,
            });
        }
        return Err(anyhow!("Expected a non-empty set of files"));
    }

    fn insert_mappings(&mut self, event: &KeyEvent) -> Result<()> {
        let cur_tab = &mut self.tabs[self.cur_tab];
        let cur_win = &mut cur_tab.windows[cur_tab.cur_win];
        let mut cur_buf = cur_win.buf.try_borrow_mut()?;

        match event.code {
            KeyCode::Char(c) => {
                cur_buf.lines[cur_win.cpos.row as usize].insert(cur_win.cpos.col as usize, c);
                cur_win.cpos.col += 1;
            }
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Backspace => {
                if let None = cur_buf.lines[cur_win.cpos.row as usize].pop() {
                    // never delete the last line
                    // TODO: Should prob wrap vector to provide this functionality for us
                    if cur_buf.lines.len() > 1 {
                        cur_buf.lines.remove(cur_win.cpos.row as usize);
                        cur_win.cpos.row -= 1;
                        cur_win.cpos.col = cur_buf.lines[cur_win.cpos.col as usize].len() as u16
                    }
                } else {
                    cur_win.cpos.col -= 1;
                }
            }
            KeyCode::Enter => {
                cur_buf
                    .lines
                    .insert(cur_win.cpos.row as usize + 1, String::new());
                cur_win.cpos.row += 1;
                cur_win.cpos.col = 0;
            }
            KeyCode::Tab => {}
            KeyCode::BackTab => {}
            KeyCode::Left => {}
            KeyCode::Right => {}
            KeyCode::Up => {}
            KeyCode::Down => {}
            _ => {}
        };
        Ok(())
    }

    fn normal_mappings(&mut self, event: &KeyEvent) -> Result<()> {
        let cur_tab = &mut self.tabs[self.cur_tab];
        let cur_win = &mut cur_tab.windows[cur_tab.cur_win];
        let cur_buf = cur_win.buf.try_borrow()?;
        match event.code {
            KeyCode::Char(c) => match c {
                'j' => {
                    cur_win.cpos.row += (cur_win.cpos.row + 1 != cur_buf.lines.len() as u16) as u16
                }
                'k' => cur_win.cpos.row -= (cur_win.cpos.row != 0) as u16,
                'l' => {
                    cur_win.cpos.col += (cur_win.cpos.col + 1
                        != cur_buf.lines[cur_win.cpos.row as usize].len() as u16)
                        as u16
                }
                'h' => cur_win.cpos.col -= (cur_win.cpos.col != 0) as u16,
                'i' => {
                    self.mode = Mode::Insert;
                }
                _ => {}
            },
            _ => {}
        };
        Ok(())
    }

    fn run(&mut self) -> Result<()> {
        let mut last_refresh = Instant::now();
        loop {
            if let Some(event) = read_key_event()? {
                if matches!(
                    event,
                    KeyEvent {
                        code: KeyCode::Char('q' | 'c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    }
                ) {
                    break Ok(());
                }
                match self.mode {
                    Mode::Normal => self.normal_mappings(&event),
                    Mode::Insert => self.insert_mappings(&event),
                }?;
            }
            if last_refresh.elapsed().as_millis() > (REFRESH_RATE_MILIS as u128) {
                self.draw()?;
                last_refresh = Instant::now();
            }
        }
    }

    fn draw(&self) -> Result<()> {
        // bufferline/tabline
        queue!(
            stderr(),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        for i in 0..self.tabs.len() {
            let s = format!("tab{}", i);
            if i == self.cur_tab {
                queue!(stderr(), Print(s.red().on_dark_blue()))?;
            } else {
                queue!(stderr(), Print(s))?;
            }
        }
        // buffer contents
        let cur_tab = &self.tabs[self.cur_tab];
        let cur_win = &cur_tab.windows[cur_tab.cur_win];
        let cur_buf = &cur_win.buf;
        for line in &cur_buf.try_borrow()?.lines {
            queue!(stderr(), Print("\r\n"), Print(line))?;
        }
        queue!(
            stderr(),
            // add 1 for our tabline/bufferline
            cursor::MoveTo(cur_win.cpos.col, cur_win.cpos.row + 1),
            cursor::Show
        )?;
        stderr().flush()?;
        Ok(())
    }
}
impl Drop for App {
    fn drop(&mut self) {
        Self::teardown().expect("Error: failed during teardown...");
    }
}

fn main() -> Result<()> {
    let mut app = App::new()?;

    app.run()?;

    Ok(())
}

fn read_key_event() -> Result<Option<KeyEvent>> {
    if event::poll(Duration::from_millis(REFRESH_RATE_MILIS))? {
        if let Event::Key(event) = event::read()? {
            return Ok(Some(event));
        }
    }
    return Ok(None);
}
