pub type Key = usize;

#[derive(Debug)]
pub enum Entry<T> {
    Full(T),
    Free(Option<Key>),
}

impl<T> Entry<T> {
    pub fn swap(&mut self, val: T) {
        debug_assert!(matches!(self, Entry::Free(_)));
        std::mem::swap(self, &mut Self::full(val))
    }

    pub fn free(idx: Option<Key>) -> Self {
        Self::Free(idx)
    }

    pub fn full(val: T) -> Self {
        Self::Full(val)
    }
}

#[derive(Debug, Default)]
pub struct Slab<T> {
    inner: Vec<Entry<T>>,
    next_idx: Option<Key>,
}

impl<T> Slab<T> {
    pub const fn new() -> Self {
        Self {
            inner: vec![],
            next_idx: None,
        }
    }

    pub fn push(&mut self, val: T) -> Key {
        match self.next_idx.take() {
            Some(idx) => {
                let entry = &mut self.inner[idx];

                let Entry::Free(next_idx) = entry else {
                    panic!("attempt to insert into a full slot.");
                };

                self.next_idx = next_idx.take();
                entry.swap(val);
                idx
            }
            None => {
                self.inner.push(Entry::full(val));
                self.inner.len() - 1
            }
        }
    }

    pub fn remove(&mut self, idx: Key) -> T {
        let mut entry = Entry::free(self.next_idx.take());
        self.next_idx = Some(idx);
        std::mem::swap(&mut self.inner[idx], &mut entry);

        match entry {
            Entry::Full(val) => val,
            Entry::Free(_) => panic!("cannot remove a free entry"),
        }
    }

    pub fn get(&self, idx: Key) -> &T {
        let Entry::Full(val) = &self.inner[idx] else {
            panic!("attempted to get an empty entry");
        };
        val
    }

    pub fn get_mut(&mut self, idx: Key) -> &mut T {
        let Entry::Full(val) = &mut self.inner[idx] else {
            panic!("attempted to get an empty entry");
        };
        val
    }

    pub fn try_get(&self, idx: Key) -> Option<&T> {
        if let Some(entry) = self.inner.get(idx) {
            let Entry::Full(val) = entry else {
                panic!("attempted to get an empty entry");
            };
            Some(val)
        } else {
            None
        }
    }

    pub fn try_get_mut(&mut self, idx: Key) -> Option<&mut T> {
        if let Some(entry) = self.inner.get_mut(idx) {
            let Entry::Full(val) = entry else {
                panic!("attempted to get an empty entry");
            };
            Some(val)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.inner
            .iter()
            .filter_map(|e| match e {
                Entry::Full(val) => Some(val),
                Entry::Free(_) => None,
            })
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn next_idx(&self) -> Key {
        match self.next_idx {
            Some(idx) => idx,
            None => self.len(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner.iter().filter_map(|e| match e {
            Entry::Full(val) => Some(val),
            Entry::Free(_) => None,
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.inner.iter_mut().filter_map(|e| match e {
            Entry::Full(val) => Some(val),
            Entry::Free(_) => None,
        })
    }
}

impl<T> IntoIterator for Slab<T> {
    type IntoIter = std::vec::IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.inner
            .into_iter()
            .filter_map(|e| match e {
                Entry::Full(val) => Some(val),
                Entry::Free(_) => None,
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}
