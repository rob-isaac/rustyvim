use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub(crate) struct WorkQueue<T> {
    queue: VecDeque<T>,
    mutex: Arc<Mutex<u32>>,
}

// TODO: Should put some bound on the size
impl<T> WorkQueue<T> {
    pub(crate) fn new() -> Self {
        WorkQueue {
            queue: VecDeque::new(),
            mutex: Arc::new(Mutex::new(0)),
        }
    }

    pub(crate) fn push_back(&mut self, val: T) {
        let lock = self.mutex.lock().unwrap();
        self.queue.push_back(val)
    }

    pub(crate) fn pop_front(&mut self) -> Option<T> {
        let lock = self.mutex.lock().unwrap();
        self.queue.pop_front()
    }

    pub(crate) fn is_empty(&self) -> bool {
        let lock = self.mutex.lock().unwrap();
        self.queue.is_empty()
    }
}
