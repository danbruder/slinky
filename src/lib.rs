use radix_fmt::{radix, Radix};

#[derive(Default, Clone)]
pub struct Merkle {
    zero: Box<Option<Merkle>>,
    one: Box<Option<Merkle>>,
    two: Box<Option<Merkle>>,
    hash: u64,
}

impl Merkle {
    fn new() -> Self {
        Default::default()
    }

    fn build(timestamps: Vec<Timestamp>) -> Self {
        let mut myself = Self::new();

        for timestamp in timestamps {
            myself = myself.insert(timestamp);
        }

        myself
    }

    fn insert(self, ts: Timestamp) -> Self {
        let hash = ts.hash();
        let key = ts.key();

        let myself = self.insert_key(key, hash);

        Self {
            hash: hash ^ myself.hash,
            ..myself
        }
    }

    fn insert_key(self, key: String, hash: u64) -> Self {
        if key.len() == 0 {
            return self;
        }

        let c = key.chars().take(1).next();
        let n = match c {
            Some('0') => self.zero.clone(),
            Some('1') => self.one.clone(),
            Some('2') => self.two.clone(),
            _ => Box::new(None),
        };
        let n: Merkle = n.unwrap_or(Merkle::new());

        let new_hash = n.hash ^ hash;
        let next = Box::new(Some(
            n.insert_key(key.chars().skip(1).collect::<String>(), hash),
        ));

        match c {
            Some('0') => Self {
                zero: next,
                hash: new_hash,
                ..self
            },
            Some('1') => Self {
                one: next,
                hash: new_hash,
                ..self
            },
            Some('2') => Self {
                two: next,
                hash: new_hash,
                ..self
            },
            _ => Merkle::new(),
        }
    }
}

struct Timestamp {
    millis: u64,
}

impl Timestamp {
    fn hash(&self) -> u64 {
        0
    }

    fn key(&self) -> String {
        format!("{}", radix(self.millis / 1000 / 60, 3))
    }
}

struct Key(String);

impl From<Key> for Timestamp {
    fn from(Key(key): Key) -> Self {
        let fullkey = format!(
            "{}{}",
            key,
            std::iter::repeat('0')
                .take(16 - key.len())
                .collect::<String>()
        );

        Timestamp {
            millis: Radix::new(fullkey, 10).parse::<u64>().unwrap() * 1000 * 60,
        }
    }
}
