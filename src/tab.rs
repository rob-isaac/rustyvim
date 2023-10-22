use crate::window::Window;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Tab {
    windows: Vec<Window>,
    cur_win: usize,
}

impl Default for Tab {
    fn default() -> Self {
        Tab {
            windows: vec![Window::new()],
            cur_win: 0,
        }
    }
}

impl Tab {
    pub(crate) fn new() -> Tab {
        Default::default()
    }

    pub(crate) fn cur_window(&self) -> &Window {
        return &self.windows[self.cur_win];
    }

    pub(crate) fn cur_window_mut(&mut self) -> &mut Window {
        return &mut self.windows[self.cur_win];
    }
}
