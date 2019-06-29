#[derive(Clone)]
pub struct NonEmpty<T>(T, Vec<T>);

#[allow(clippy::len_without_is_empty)]
impl<T> NonEmpty<T> {
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
}

impl<T> Into<Vec<T>> for NonEmpty<T> {
    /// Turns a non-empty list into a Vec.
    fn into(self) -> Vec<T> {
        std::iter::once(self.0).chain(self.1).collect()
    }
}
