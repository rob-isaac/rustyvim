use crate::buffer::{Buffer, Mark};
use crate::buffer_list::BufferList;
use crate::pane_manager::PaneManager;

// TODO: Add more modes + support user-defined modes

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
}

pub struct Application {
    buffer_manager: BufferList,
    pane_manager: PaneManager,
    mode: Mode,
}

impl Application {
    pub fn new() -> Self {
        let mut app = Self {
            buffer_manager: BufferList::new(),
            pane_manager: PaneManager::new(),
            mode: Mode::Normal,
        };
        app.buffer_manager.add_buf(Buffer::new());
        app
    }

    pub fn get_active_buf(&self) -> &Buffer {
        let cur_pane = self.pane_manager.get_active_pane();
        self.buffer_manager.get_buf(cur_pane.buf_num).unwrap()
    }

    pub fn get_active_buf_mut(&mut self) -> &mut Buffer {
        let cur_pane = self.pane_manager.get_active_pane();
        self.buffer_manager.get_buf_mut(cur_pane.buf_num).unwrap()
    }

    pub fn get_active_mark(&self) -> &Mark {
        let cur_pane_num = self.pane_manager.get_active_pane_num();
        self.get_active_buf().get_mark(cur_pane_num)
    }

    pub fn get_active_mark_mut(&mut self) -> &mut Mark {
        let cur_pane_num = self.pane_manager.get_active_pane_num();
        self.get_active_buf_mut().get_mark_mut(cur_pane_num)
    }

    pub fn get_mode(&self) -> Mode {
        self.mode
    }
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
}
