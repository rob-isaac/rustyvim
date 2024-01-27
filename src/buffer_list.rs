use crate::buffer::Buffer;

pub struct BufferList {
    buffers: Vec<Option<Buffer>>,
}

impl BufferList {
    pub fn new() -> Self {
        BufferList { buffers: vec![] }
    }
    pub fn from_bufs(bufs: Vec<Buffer>) -> Self {
        BufferList {
            buffers: bufs.into_iter().map(|e| Some(e)).collect(),
        }
    }
    pub fn num_bufs(&self) -> usize {
        self.buffers
            .iter()
            .fold(0, |acc, e| acc + (e.is_some() as usize))
    }
    pub fn add_buf(&mut self, buf: Buffer) -> usize {
        self.buffers.push(Some(buf));
        self.buffers.len() - 1
    }
    pub fn get_buf(&self, idx: usize) -> Option<&Buffer> {
        self.buffers[idx].as_ref()
    }
    pub fn get_buf_mut(&mut self, idx: usize) -> Option<&mut Buffer> {
        self.buffers[idx].as_mut()
    }
    pub fn remove(&mut self, idx: usize) {
        self.buffers[idx] = None;
    }
    pub fn iter(&self) -> impl Iterator<Item = &Buffer> {
        self.buffers.iter().filter_map(|e| e.as_ref())
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Buffer> {
        self.buffers.iter_mut().filter_map(|e| e.as_mut())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_empty() {
        assert_eq!(BufferList::new().num_bufs(), 0);
    }
    #[test]
    fn create_from_buffers() {
        assert_eq!(
            BufferList::from_bufs(vec![Buffer::new(), Buffer::new()]).num_bufs(),
            2
        );
    }
    #[test]
    fn add_buffer() {
        let mut list = BufferList::new();
        let bnum = list.add_buf(Buffer::from_str("hello world"));
        assert_eq!(list.num_bufs(), 1);
        assert_eq!(list.get_buf(bnum).unwrap().to_str(), "hello world");
    }
    #[test]
    fn mutate_buffer() {
        let mut list = BufferList::new();
        let bnum = list.add_buf(Buffer::new());
        list.get_buf_mut(bnum)
            .unwrap()
            .insert_str(0, 0, "hello world");
        assert_eq!(list.num_bufs(), 1);
        assert_eq!(list.get_buf(bnum).unwrap().to_str(), "hello world");
    }
    #[test]
    fn remove_buffer() {
        let mut list = BufferList::from_bufs(vec![Buffer::new()]);
        list.remove(0);
        assert_eq!(list.num_bufs(), 0);
    }
    #[test]
    fn iteration() {
        let mut list = BufferList::from_bufs(vec![Buffer::from_str("hi"), Buffer::from_str("bye")]);
        list.remove(0);
        for buf in list.iter() {
            assert_eq!(buf.to_str(), "bye");
        }
    }
    #[test]
    fn mutable_iteration() {
        let mut list = BufferList::from_bufs(vec![Buffer::from_str("hi"), Buffer::from_str("bye")]);
        list.remove(0);
        for buf in list.iter_mut() {
            buf.insert_str(0, 0, "hi ");
            assert_eq!(buf.to_str(), "hi bye");
        }
    }
}
