use std::{io::Write, sync::mpsc, time::Instant};

use crate::{
    draw_info::DrawInfo,
    events::{Event, Key},
    tab::Tab,
};
use anyhow::Result;

const DEFAULT_REFRESH_RATE_MS: u64 = 30;

#[derive(Debug, PartialEq, Eq)]
enum Mode {
    Normal,
    Insert,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ScreenSize {
    pub(crate) rows: usize,
    pub(crate) cols: usize,
}

#[derive(Debug)]
struct UISubscriber {
    ui_chan: mpsc::Sender<DrawInfo>,
    screen_size: ScreenSize,
}

#[derive(Debug)]
pub(crate) struct AppServer {
    tabs: Vec<Tab>,
    cur_tab: usize,
    mode: Mode,
    events: mpsc::Receiver<Event>,
    ui_subscriber: UISubscriber,
}

impl AppServer {
    pub(crate) fn new(
        event_rx: mpsc::Receiver<Event>,
        ui_tx: mpsc::Sender<DrawInfo>,
        screen_size: ScreenSize,
    ) -> Self {
        AppServer {
            tabs: vec![Tab::new()],
            cur_tab: 0,
            mode: Mode::Normal,
            events: event_rx,
            ui_subscriber: UISubscriber {
                ui_chan: ui_tx,
                screen_size,
            },
        }
    }

    fn normal_mappings(&mut self, key: Key) -> Result<()> {
        let cur_tab = &mut self.tabs[self.cur_tab];
        let cur_win = cur_tab.cur_window_mut();
        let mut cpos = cur_win.cursor_pos();
        {
            let cur_buf = cur_win.cur_buffer_mut();
            match key {
                Key::Char(c) => match c {
                    'j' => cpos.row += (cpos.row + 1 != cur_buf.num_lines()) as usize,
                    'k' => cpos.row -= (cpos.row != 0) as usize,
                    'l' => cpos.col += (cpos.col + 1 != cur_buf.line_len(cpos.row)) as usize,
                    'h' => cpos.col -= (cpos.col != 0) as usize,
                    'i' => self.mode = Mode::Insert,
                    _ => {}
                },
                _ => {}
            };
        }
        cur_win.set_cursor_pos(cpos);
        Ok(())
    }

    fn insert_mappings(&mut self, key: Key) -> Result<()> {
        let cur_tab = &mut self.tabs[self.cur_tab];
        let cur_win = cur_tab.cur_window_mut();
        let mut cpos = cur_win.cursor_pos();

        {
            let cur_buf = &mut cur_win.cur_buffer_mut();
            match key {
                Key::Char(c) => {
                    cur_buf.insert(cpos.row, cpos.col, c);
                    cpos.col += 1;
                }
                Key::Esc => self.mode = Mode::Normal,
                Key::Backspace => {
                    if cpos.col != 0 {
                        cpos.col -= 1;
                        cur_buf.remove(cpos.row, cpos.col);
                    } else if cpos.row != 0 {
                        cpos.row -= 1;
                        cpos.col = cur_buf.line_len(cpos.row);
                        cur_buf.join_below(cpos.row)
                    }
                }
                Key::Enter => {
                    cur_buf.insert_line(cpos.row);
                    cpos.row += 1;
                    cpos.col = 0;
                }
                Key::Tab => {}
                Key::BackTab => {}
                Key::Left => {}
                Key::Right => {}
                Key::Up => {}
                Key::Down => {}
                _ => {}
            };
        }
        cur_win.set_cursor_pos(cpos);
        Ok(())
    }

    fn make_draw_info(&self) -> Result<DrawInfo> {
        // Tabline
        let mut lines = vec![(0..self.tabs.len())
            .map(|i| format!("tab{}", i))
            .collect::<Vec<_>>()
            .join(" ")];

        let cur_tab = &self.tabs[self.cur_tab];
        let cur_win = &cur_tab.cur_window();
        let cur_buf = &cur_win.cur_buffer();
        let cpos = cur_win.cursor_pos();
        let win_offset = cur_win.window_offset();
        let ui_size = &self.ui_subscriber.screen_size;

        // Windows
        for line in &cur_buf.lines()[win_offset.row..win_offset.row + ui_size.rows - 1] {
            lines.push(line[..ui_size.cols].to_string());
        }

        Ok(DrawInfo {
            lines,
            cpos: (cpos.row - win_offset.row, cpos.col - win_offset.col),
        })
    }

    pub(crate) fn serve_loop(&mut self) -> Result<()> {
        let mut last_draw = Instant::now();
        loop {
            let request = self.events.recv().unwrap();

            match request {
                Event::Kill => {
                    return Ok(());
                }
                Event::ResizeUI(_) => {
                    todo!()
                }
                Event::KeyPress(key) => {
                    match self.mode {
                        Mode::Normal => self.normal_mappings(key),
                        Mode::Insert => self.insert_mappings(key),
                    }?;
                }
            }

            if last_draw.elapsed().as_millis() > (DEFAULT_REFRESH_RATE_MS as u128) {
                self.ui_subscriber.ui_chan.send(self.make_draw_info()?);
                last_draw = Instant::now();
            }
        }
    }
}
