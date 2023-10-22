pub(crate) enum Event {
    KeyPress(Key),
    ResizeUI(UISize),
    Kill,
}

pub(crate) enum Key {
    Char(char),
    Esc,
    Backspace,
    Enter,
    Space,
    Tab,
    BackTab,
    Up,
    Down,
    Left,
    Right,
}

pub(crate) struct UISize(usize, usize);
