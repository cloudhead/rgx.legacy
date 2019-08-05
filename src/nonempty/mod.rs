#[derive(Clone, Debug)]
pub struct NonEmpty<T>(T, Vec<T>);

#[allow(clippy::len_without_is_empty)]
impl<T> NonEmpty<T> {
    pub fn new(e: T) -> Self {
        Self::singleton(e)
    }

    pub fn singleton(e: T) -> Self {
        NonEmpty(e, Vec::new())
    }

    pub fn push(&mut self, e: T) {
        self.1.push(e)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.1.pop()
    }

    pub fn len(&self) -> usize {
        self.1.len() + 1
    }

    pub fn first(&self) -> &T {
        &self.0
    }

    pub fn last(&self) -> &T {
        match self.1.last() {
            None => &self.0,
            Some(e) => e,
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index == 0 {
            Some(&self.0)
        } else {
            self.1.get(index - 1)
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index == 0 {
            Some(&mut self.0)
        } else {
            self.1.get_mut(index - 1)
        }
    }

    pub fn truncate(&mut self, len: usize) {
        assert!(len >= 1);
        self.1.truncate(len - 1);
    }
}

impl<T> Into<Vec<T>> for NonEmpty<T> {
    /// Turns a non-empty list into a Vec.
    fn into(self) -> Vec<T> {
        std::iter::once(self.0).chain(self.1).collect()
    }
}
