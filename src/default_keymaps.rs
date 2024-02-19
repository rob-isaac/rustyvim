use crate::{
    application::{Application, Mode},
    buffer::Mark,
    keymaps::Trie,
};

const ESCAPE_STR: &str = "\u{001b}";
const BACKSPACE_STR: &str = "\u{0008}";

pub fn make_default_keymaps(mode: Mode) -> Trie<Application> {
    let mut keymaps = Trie::new();

    match mode {
        Mode::Normal => {
            keymaps.insert(
                "j",
                Box::new(|app: &mut Application| {
                    let n_lines = app.get_active_buf().get_lines().len();
                    debug_assert!(n_lines > 0);
                    app.get_active_mark_mut().move_row(1).clamp_row(n_lines - 1);
                }),
            );
            keymaps.insert(
                "k",
                Box::new(|app: &mut Application| {
                    app.get_active_mark_mut().move_row(-1);
                }),
            );
            keymaps.insert(
                "l",
                Box::new(|app: &mut Application| {
                    let n_lines = app.get_active_buf().get_lines().len();
                    let cur_row = app.get_active_mark().row;
                    debug_assert!(n_lines > 0);
                    debug_assert!(cur_row < n_lines);
                    let n_cols = app.get_active_buf().get_lines()[cur_row].len();
                    app.get_active_mark_mut().move_col(1).clamp_col(n_cols);
                }),
            );
            keymaps.insert(
                "h",
                Box::new(|app: &mut Application| {
                    app.get_active_mark_mut().move_col(-1);
                }),
            );
            keymaps.insert(
                "i",
                Box::new(|app: &mut Application| {
                    app.set_mode(Mode::Insert);
                }),
            );
        }
        Mode::Insert => {
            for c in ('a'..='z').chain('A'..='Z').chain('0'..='9') {
                let s_key = c.to_string();
                let s_capture = c.to_string();
                keymaps.insert(
                    &s_key,
                    Box::new(move |app: &mut Application| {
                        let mark = app.get_active_mark().clone();
                        app.get_active_buf_mut().insert(mark, &s_capture);
                        app.get_active_mark_mut().move_col(1);
                    }),
                );
            }
            keymaps.insert(
                "\n",
                Box::new(|app: &mut Application| {
                    let mark = app.get_active_mark().clone();
                    app.get_active_buf_mut().insert(mark, "\n");
                    app.get_active_mark_mut().move_row(1).clamp_col(0);
                }),
            );
            keymaps.insert(
                ESCAPE_STR,
                Box::new(|app: &mut Application| {
                    app.set_mode(Mode::Normal);
                }),
            );
            keymaps.insert(
                BACKSPACE_STR,
                Box::new(|app: &mut Application| {
                    let mark_end = app.get_active_mark().clone();
                    if mark_end.col > 0 {
                        let mark_start = Mark::new(mark_end.row, mark_end.col - 1);
                        app.get_active_buf_mut().remove(mark_start, mark_end);
                    } else if mark_end.row > 0 {
                        let prev_line_len =
                            app.get_active_buf().get_lines()[mark_end.row - 1].len();
                        let mark_start = Mark::new(mark_end.row - 1, prev_line_len);
                        app.get_active_buf_mut().remove(mark_start, mark_end);
                    }
                }),
            );
        }
    };

    keymaps
}

#[cfg(test)]
mod tests {
    use crate::buffer::Mark;

    use super::*;

    #[test]
    fn insert_a2z() {
        let mut app = Application::new();
        let keymaps = make_default_keymaps(Mode::Insert);
        (keymaps.get("a").unwrap())(&mut app);
        (keymaps.get("b").unwrap())(&mut app);
        (keymaps.get("z").unwrap())(&mut app);
        (keymaps.get("\n").unwrap())(&mut app);
        (keymaps.get("D").unwrap())(&mut app);
        (keymaps.get("E").unwrap())(&mut app);
        (keymaps.get("Z").unwrap())(&mut app);
        (keymaps.get("\n").unwrap())(&mut app);
        (keymaps.get("0").unwrap())(&mut app);
        (keymaps.get("1").unwrap())(&mut app);
        (keymaps.get("9").unwrap())(&mut app);
        assert_eq!(app.get_active_buf().to_str(), "abz\nDEZ\n019");
    }

    #[test]
    fn normal_hjkl() {
        let mut app = Application::new();
        let keymaps = make_default_keymaps(Mode::Normal);

        app.get_active_buf_mut()
            .insert(Mark::new(0, 0), "abz\nDEZ\n019");

        assert_eq!(*app.get_active_mark(), Mark::new(0, 0));
        (keymaps.get("j").unwrap())(&mut app);
        assert_eq!(*app.get_active_mark(), Mark::new(1, 0));
        (keymaps.get("k").unwrap())(&mut app);
        assert_eq!(*app.get_active_mark(), Mark::new(0, 0));
        (keymaps.get("k").unwrap())(&mut app);
        assert_eq!(*app.get_active_mark(), Mark::new(0, 0));
        (keymaps.get("l").unwrap())(&mut app);
        assert_eq!(*app.get_active_mark(), Mark::new(0, 1));
        (keymaps.get("h").unwrap())(&mut app);
        assert_eq!(*app.get_active_mark(), Mark::new(0, 0));
        (keymaps.get("h").unwrap())(&mut app);
        assert_eq!(*app.get_active_mark(), Mark::new(0, 0));
        for _ in 0..5 {
            (keymaps.get("j").unwrap())(&mut app);
            (keymaps.get("l").unwrap())(&mut app);
        }
        assert_eq!(*app.get_active_mark(), Mark::new(2, 3));
    }

    #[test]
    fn swap_modes() {
        let mut app = Application::new();
        let normal_keymaps = make_default_keymaps(Mode::Normal);
        let insert_keymaps = make_default_keymaps(Mode::Insert);

        assert_eq!(app.get_mode(), Mode::Normal);
        (normal_keymaps.get("i").unwrap())(&mut app);
        assert_eq!(app.get_mode(), Mode::Insert);
        (insert_keymaps.get(ESCAPE_STR).unwrap())(&mut app);
        assert_eq!(app.get_mode(), Mode::Normal);
    }

    #[test]
    fn backspace() {
        let mut app = Application::new();
        let keymaps = make_default_keymaps(Mode::Insert);

        app.get_active_buf_mut()
            .insert(Mark::new(0, 0), "abz\nDEZ\n019");
        *app.get_active_mark_mut() = Mark::new(1, 0);

        (keymaps.get(BACKSPACE_STR).unwrap())(&mut app);
        (keymaps.get(BACKSPACE_STR).unwrap())(&mut app);
        assert_eq!(app.get_active_buf().to_str(), "abDEZ\n019");
        assert_eq!(*app.get_active_mark(), Mark::new(0, 2));
    }
}
