use std::{cell::RefCell, rc::Rc};

use crate::buffer::Buffer;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub(crate) struct CursorPos {
    pub(crate) row: usize,
    pub(crate) col: usize,
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub(crate) struct WindowOffset {
    pub(crate) row: usize,
    pub(crate) col: usize,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub(crate) struct Window {
    buf: Rc<RefCell<Buffer>>,
    cpos: CursorPos,
    win_offset: WindowOffset,
}

impl Window {
    pub(crate) fn new() -> Window {
        Default::default()
    }

    pub(crate) fn cur_buffer(&self) -> std::cell::Ref<'_, Buffer> {
        self.buf.borrow()
    }

    pub(crate) fn cur_buffer_mut(&mut self) -> std::cell::RefMut<'_, Buffer> {
        self.buf.borrow_mut()
    }

    // FIXME: remove clone
    pub(crate) fn cursor_pos(&self) -> CursorPos {
        self.cpos.clone()
    }

    pub(crate) fn set_cursor_pos(&mut self, cpos: CursorPos) {
        self.cpos = cpos;
    }

    // FIXME: remove clone
    pub(crate) fn window_offset(&self) -> WindowOffset {
        self.win_offset.clone()
    }

    pub(crate) fn set_window_offset(&mut self, win_offset: WindowOffset) {
        self.win_offset = win_offset;
    }
}
