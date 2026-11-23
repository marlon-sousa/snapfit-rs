//! Tills written by a `macro_rules!` whose fallible ledger reaches the macro
//! as tokens: inside the whole struct, or as the `fallible` word alone.

use std::sync::Arc;

use snapfit::container;

use super::Ledger;

/// Adds `#[container]` to a struct it is passed whole.
macro_rules! whole {
    ($($item:tt)*) => {
        #[container]
        $($item)*
    };
}

/// Writes a till whose ledger's `fallible` word it is passed.
macro_rules! word_till {
    ($till:ident, $word:ident) => {
        #[container]
        pub struct $till {
            #[singleton($word = true)]
            ledger: Arc<Ledger>,
        }
    };
}

whole! {
    pub struct WholeTill {
        #[singleton(fallible = true)]
        ledger: Arc<Ledger>,
    }
}

word_till!(WordTill, fallible);
