#[derive(Default, Debug, PartialEq, Eq)]
pub(crate) struct DrawInfo {
    pub(crate) lines: Vec<String>,
    pub(crate) cpos: (usize, usize),
}
