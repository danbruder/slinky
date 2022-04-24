use chrono::{TimeZone, Utc};
use std::char::from_digit;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

use fasthash::{murmur3, Murmur3Hasher};
use radix_fmt::radix;

#[derive(Default, PartialEq, Debug, Clone)]
pub struct Merkle {
    zero: Node,
    one: Node,
    two: Node,
    hash: u32,
}

#[derive(Default, PartialEq, Debug, Clone)]
struct Node(Option<Box<Merkle>>);

impl Merkle {
    fn new() -> Self {
        Default::default()
    }
}

fn insert(trie: Merkle, ts: Timestamp) -> Merkle {
    let hash = ts.hash();
    let key = ts.into();

    let Node(node) = insert_key(
        Node(Some(Box::new(Merkle {
            hash: trie.hash ^ hash,
            ..trie
        }))),
        key,
        hash,
    );

    *node.unwrap_or_default()
}

// Need some tests!
fn insert_key(&mut trie: Merkle, key: Key, hash: u32) -> Merkle {
    let mut key = key;

    match key.pop_front() {
        Some(0) => Merkle {
            zero: insert_key(trie.zero, key, hash), // Need to hash here
            ..trie
        },
        Some(1) => Merkle {
            one: insert_key(trie.one, key, hash),
            ..trie
        },
        Some(2) => Merkle {
            two: insert_key(trie.two, key, hash),
            ..trie
        },
        _ => trie, // Final case
    }
}

#[derive(Debug, PartialEq)]
struct Timestamp {
    millis: u64,
    counter: u64,
    node: String,
}

impl Timestamp {
    fn from_millis(millis: u64) -> Self {
        Self {
            millis,
            counter: 0,
            node: "".into(),
        }
    }

    fn hash(&self) -> u32 {
        let str_self = self.to_string();
        murmur3::hash32(&str_self)
    }
}

impl ToString for Timestamp {
    fn to_string(&self) -> String {
        let ts = Utc.timestamp_millis(self.millis as i64).to_rfc3339();
        let counter = format!("{:0>4}", radix(self.counter, 16)).to_uppercase();
        let node = format!("{:0>16}", self.node);

        vec![ts, counter, node].join("-")
    }
}

#[derive(PartialEq, Debug)]
struct Key(VecDeque<u32>);

impl Key {
    fn from_millis(millis: u64) -> Self {
        let key = format!("{}", radix(millis / 1000 / 60, 3));
        Self::from_base3_str(&key)
    }

    fn to_millis(self) -> u64 {
        let key = self
            .0
            .into_iter()
            .map(|i| i.to_string())
            .collect::<String>();
        u64::from_str_radix(&key, 3).expect("Invalid key")
    }

    fn from_base3_str(base3_str: &str) -> Self {
        let key = format!("{}", base3_str);
        let key = key
            .chars()
            .map(|c| u32::from_str_radix(&c.to_string(), 10))
            .collect::<Result<VecDeque<_>, _>>()
            .expect("Invalid");
        Key(key)
    }

    fn pop_front(&mut self) -> Option<u32> {
        let Key(k) = self;
        k.pop_front()
    }
}

impl From<Timestamp> for Key {
    fn from(ts: Timestamp) -> Self {
        Key::from_millis(ts.millis)
    }
}

impl From<Key> for Timestamp {
    fn from(key: Key) -> Self {
        Timestamp::from_millis(key.to_millis())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn key_pop_front() {
        let mut key = Key::from_base3_str("123");
        let got = key.pop_front();
        let want = Some(1);
        assert_eq!(got, want);
    }

    #[test]
    fn ts_to_key() {
        let ts = Timestamp::from_millis(10 * 1000 * 60);
        let got: Key = ts.into();
        let want = Key::from_base3_str("101");
        assert_eq!(got, want);
    }

    #[test]
    fn key_to_ts() {
        let key = Key::from_base3_str("101");
        let got: Timestamp = key.into();
        let want = Timestamp::from_millis(10);

        assert_eq!(got, want);
    }

    #[test]
    fn insert_test() {
        let tree1 = Merkle::new();
        let ts1 = Timestamp::from_millis(10);

        let got = insert(tree1, ts1);
        let want = Merkle {
            zero: Node(Some(Box::new(Merkle {
                zero: Node(None),
                one: Node(None),
                two: Node(None),
                hash: 0,
            }))),
            one: Node(None),
            two: Node(None),
            hash: 1402235473,
        };

        assert_eq!(got, want);
    }
}
