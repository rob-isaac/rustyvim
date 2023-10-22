use std::{
    io::{stderr, Write},
    sync::mpsc::{Receiver, Sender},
    time::{Duration, Instant},
};

use crate::{
    app_server::{AppServer, ScreenSize},
    draw_info::DrawInfo,
};
use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, KeyCode, KeyEvent, KeyEventKind, KeyEventState},
    execute, queue,
    style::Print,
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, EnterAlternateScreen},
};

use std::sync::mpsc;

const DEFAULT_KEY_POLL_RATE_MS: u64 = 5;

pub struct TUI {
    out: Box<dyn Write>,
    app: AppServer,
    event_channel: Sender<crate::events::Event>,
    ui_channel: Receiver<DrawInfo>,
}

impl Drop for TUI {
    fn drop(&mut self) {
        let r1 = disable_raw_mode();
        let r2 = execute!(
            stderr(),
            Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        );
        self.event_channel.send(crate::events::Event::Kill);
        r1.unwrap();
        r2.unwrap();
    }
}

impl TUI {
    pub fn new() -> Result<TUI> {
        let mut out = stderr();
        execute!(out, EnterAlternateScreen)?;
        enable_raw_mode()?;
        queue!(out, cursor::MoveTo(0, 0), cursor::Show)?;
        let (event_tx, event_rx) = mpsc::channel();
        let (ui_tx, ui_rx) = mpsc::channel();
        let win_size = terminal::window_size()?;
        let mut tui = TUI {
            out: Box::new(out),
            app: AppServer::new(
                event_rx,
                ui_tx,
                ScreenSize {
                    rows: win_size.rows as usize,
                    cols: win_size.columns as usize,
                },
            ),
            event_channel: event_tx,
            ui_channel: ui_rx,
        };
        tui.app.serve_loop()?;
        Ok(tui)
    }

    fn draw(&mut self, draw_info: DrawInfo) -> Result<()> {
        // bufferline/tabline
        queue!(
            self.out,
            // TODO: I think don't actually need to clear here b/c we overwrite everything.
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        for line in &draw_info.lines[..draw_info.lines.len() - 1] {
            queue!(self.out, Print(line), Print("\r\n"))?;
        }
        if let Some(last_line) = draw_info.lines.last() {
            queue!(self.out, Print(last_line));
        }
        queue!(
            stderr(),
            cursor::MoveTo(draw_info.cpos.0 as u16, draw_info.cpos.1 as u16),
            cursor::Show
        )?;
        stderr().flush()?;
        Ok(())
    }

    pub fn event_loop(&mut self) -> Result<()> {
        let mut last_refresh = Instant::now();
        loop {
            if let Ok(draw_info) = self.ui_channel.try_recv() {
                let mut draw_info = draw_info;
                while let Ok(next_draw_info) = self.ui_channel.try_recv() {
                    draw_info = next_draw_info;
                }
            }

            // if let Some(event) = read_key_event()? {
            //     self.event_channel.send(AppServerRequest::KeyEvent(event))?;
            // }
            // if last_refresh.elapsed().as_millis() > (DEFAULT_REFRESH_RATE_MS as u128) {
            //     self.draw()?;
            //     last_refresh = Instant::now();
            // }
        }
    }
}

fn read_key_event() -> Result<Option<KeyEvent>> {
    if event::poll(Duration::from_millis(DEFAULT_KEY_POLL_RATE_MS))? {
        if let event::Event::Key(event) = event::read()? {
            return Ok(Some(event));
        }
    }
    Ok(None)
}
