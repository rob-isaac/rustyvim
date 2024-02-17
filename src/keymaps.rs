use anyhow::{anyhow, Result};

// TODO: support more than just a..zA..Z
// TODO: we should make most things in here private and only export Trie<Application>
// TODO: make root node a different type which doesn't have an associated function

const NUM_KEYS: usize = 2 * 26;

type Op<T> = Box<dyn Fn(&mut T) -> ()>;

struct TrieNode<T> {
    op: Op<T>,
    next: [Option<Box<TrieNode<T>>>; NUM_KEYS],
}

pub struct Trie<T> {
    root: TrieNode<T>,
}

impl<T> TrieNode<T> {
    fn new(op: Op<T>) -> Self {
        Self {
            op,
            next: std::array::from_fn(|_| None),
        }
    }
}

impl<T> Trie<T> {
    pub fn new() -> Self {
        Self {
            root: TrieNode::new(Box::new(|app| ())),
        }
    }
    fn get(&self, key: &str) -> Option<&TrieNode<T>> {
        let mut cur = &self.root;
        for char in key.chars() {
            if let Some(next) = match char {
                'a'..='z' => cur.next[char.to_digit(26).unwrap() as usize].as_ref(),
                'A'..='Z' => cur.next[26 + char.to_digit(26).unwrap() as usize].as_ref(),
                _ => None,
            } {
                cur = next;
            } else {
                return None;
            }
        }
        Some(cur)
    }
    fn insert(&mut self, key: &str, op: Op<T>) -> Result<()> {
        let mut cur = &mut self.root;
        for char in key.chars() {
            let next = match char {
                'a'..='z' => Ok(&mut cur.next[char.to_digit(26).unwrap() as usize]),
                'A'..='Z' => Ok(&mut cur.next[26 + char.to_digit(26).unwrap() as usize]),
                _ => Err(anyhow!("Unsupported key {}", char)),
            }?;
            if next.is_none() {
                *next = Some(Box::new(TrieNode::new(Box::new(|_| ()))));
            }
            cur = next.as_mut().unwrap();
        }
        cur.op = op;
        Ok(())
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
        let mut app = FakeApp {state: 0};

        assert!(trie.get("a").is_none());
        trie.insert("a", Box::new(|app: &mut FakeApp| {app.state = 1;})).unwrap();
        assert!(trie.get("b").is_none());

        (trie.get("a").unwrap().op)(&mut app);
        assert_eq!(app.state, 1);
    }

    #[test]
    fn get_layer2() {
        let mut trie = Trie::new();
        let mut app = FakeApp {state: 0};

        assert!(trie.get("ab").is_none());
        trie.insert("ab", Box::new(|app: &mut FakeApp| {app.state = 2;})).unwrap();
        assert!(trie.get("ac").is_none());
        
        (trie.get("a").unwrap().op)(&mut app);
        assert_eq!(app.state, 0);
        (trie.get("ab").unwrap().op)(&mut app);
        assert_eq!(app.state, 2);
    }
}
