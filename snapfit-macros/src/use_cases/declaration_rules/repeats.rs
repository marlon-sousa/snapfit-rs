//! Policy: the items of a list that come more than once, for the rules of
//! its parent.

use std::collections::HashSet;
use std::hash::Hash;

use derive_new::new;

pub(super) fn repeats<T: Clone + Eq + Hash>(items: impl Iterator<Item = T>) -> Repeats<T> {
    items.fold(Repeats::new(), Repeats::with)
}

/// The items read so far: each once in the order it first came, and each
/// that came again, once, in the order of its second coming.
#[derive(new)]
pub(super) struct Repeats<T> {
    #[new(default)]
    seen: HashSet<T>,
    #[new(default)]
    pub(super) distinct: Vec<T>,
    #[new(default)]
    pub(super) repeated: Vec<T>,
}

impl<T: Clone + Eq + Hash> Repeats<T> {
    fn with(mut self, item: T) -> Self {
        if self.seen.insert(item.clone()) {
            self.distinct.push(item);
        } else if !self.repeated.contains(&item) {
            self.repeated.push(item);
        }
        self
    }
}
