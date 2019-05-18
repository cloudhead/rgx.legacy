#[derive(Clone)]
struct NonEmpty<T>(T, Vec<T>);

impl<T> NonEmpty<T>
where
    T: Clone,
{
    fn singleton(e: T) -> Self {
        NonEmpty(e, Vec::new())
    }

    fn push(&mut self, e: T) {
        self.1.push(e)
    }

    fn pop(&mut self) -> Option<T> {
        self.1.pop()
    }

    fn len(&self) -> usize {
        self.1.len() + 1
    }

    fn last(&self) -> &T {
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
