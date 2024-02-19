use std::collections::HashMap;

// TODO: we should make most things in here private and only export Trie<Application>
// TODO: make root node a different type which doesn't have an associated function
// TODO: how to handle number-repeats? Can start by juts mapping <num> -> for 0..<num>
// TODO: use better representation than nested hashmaps

pub type Op<T> = Box<dyn Fn(&mut T) -> ()>;

struct TrieNode<T> {
    op: Op<T>,
    next: HashMap<char, Box<TrieNode<T>>>,
}

pub struct Trie<T> {
    root: TrieNode<T>,
}

impl<T> TrieNode<T> {
    fn new(op: Op<T>) -> Self {
        Self {
            op,
            next: HashMap::new(),
        }
    }
}

impl<T> Trie<T> {
    pub fn new() -> Self {
        Self {
            root: TrieNode::new(Box::new(|_| ())),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Op<T>> {
        let mut cur = &self.root;
        for char in key.chars() {
            if let Some(next) = cur.next.get(&char) {
                cur = next;
            } else {
                return None;
            }
        }
        Some(&cur.op)
    }
    pub fn insert(&mut self, key: &str, op: Op<T>) {
        let mut cur = &mut self.root;
        for char in key.chars() {
            cur = cur.next.entry(char).or_insert(Box::new(TrieNode::new(Box::new(|_|()))));
        }
        cur.op = op;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeApp {
        state: usize,
    }

    #[test]
    fn get_layer1() {
        let mut trie = Trie::new();
        let mut app = FakeApp { state: 0 };

        assert!(trie.get("a").is_none());
        trie.insert(
            "a",
            Box::new(|app: &mut FakeApp| {
                app.state = 1;
            }),
        );
        assert!(trie.get("b").is_none());

        (trie.get("a").unwrap())(&mut app);
        assert_eq!(app.state, 1);
    }

    #[test]
    fn get_layer2() {
        let mut trie = Trie::new();
        let mut app = FakeApp { state: 0 };

        assert!(trie.get("ab").is_none());
        trie.insert(
            "ab",
            Box::new(|app: &mut FakeApp| {
                app.state = 2;
            }),
        );
        assert!(trie.get("ac").is_none());

        (trie.get("a").unwrap())(&mut app);
        assert_eq!(app.state, 0);
        (trie.get("ab").unwrap())(&mut app);
        assert_eq!(app.state, 2);
    }
}
