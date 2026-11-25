//! Policy: the implementations a dispatched type holds, each once by how it
//! is written, a `self::` that starts any of its paths aside, and every
//! other spelling of one it holds.

use derive_new::new;

use crate::domain::{Respelling, Variant};

/// The implementations, in the order first named, and their other spellings.
#[derive(new)]
pub(super) struct HeldOnce<T> {
    #[new(default)]
    pub(super) variants: Vec<Variant<T>>,
    #[new(default)]
    pub(super) respelled: Vec<Respelling<T>>,
}

pub(super) fn held_once<'a, T: Clone + 'a>(
    named: impl Iterator<Item = &'a Variant<T>>,
) -> HeldOnce<T> {
    named.fold(HeldOnce::new(), HeldOnce::with)
}

impl<T: Clone> HeldOnce<T> {
    fn with(mut self, variant: &Variant<T>) -> Self {
        let spelling = variant.name.without_self_paths();
        let held = self
            .variants
            .iter()
            .find(|held| held.name.without_self_paths() == spelling);
        match held {
            None => self.variants.push(variant.clone()),
            Some(held) if held.name != variant.name => {
                let respelling = Respelling::new(held.syntax.clone(), variant.syntax.clone());
                self.respelled.push(respelling);
            }
            Some(_) => {}
        }
        self
    }
}
