use std::{cell::RefCell, rc::Rc};

use crate::buffer::Buffer;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub(crate) struct WindowOffset {
    pub(crate) row: usize,
    pub(crate) col: usize,
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub(crate) struct WindowId(pub(crate) usize);

#[derive(Default, Debug, PartialEq, Eq)]
pub(crate) struct Window {
    win_id: WindowId,
    buf: Rc<RefCell<Buffer>>,
    win_offset: WindowOffset,
}

impl Window {
    pub(crate) fn new() -> Window {
        Default::default()
    }

    pub(crate) fn id(&self) -> WindowId {
        self.win_id
    }

    pub(crate) fn cur_buffer(&self) -> std::cell::Ref<'_, Buffer> {
        self.buf.borrow()
    }

    pub(crate) fn cur_buffer_mut(&mut self) -> std::cell::RefMut<'_, Buffer> {
        self.buf.borrow_mut()
    }

    pub(crate) fn window_offset(&self) -> &WindowOffset {
        &self.win_offset
    }

    pub(crate) fn set_window_offset(&mut self, win_offset: WindowOffset) {
        self.win_offset = win_offset;
    }
}
