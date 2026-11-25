//! Containers that write one implementation both with and without `self::`:
//! inside a type argument, through a `macro_rules!`, and inside a function,
//! where both spellings still name one type.

use std::marker::PhantomData;
use std::sync::Arc;

use snapfit::container;

use super::ledgers::Ledger;

/// A shelf of one kind of record.
pub struct Shelf<T>(PhantomData<T>);

impl<T> Shelf<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

pub struct Paper;

impl Ledger for Shelf<Paper> {
    fn source(&self) -> String {
        "shelf of paper".to_owned()
    }
}

/// Lists the type it is given beside a field that writes `Shelf<Paper>`.
macro_rules! shelved {
    ($listed:ty) => {
        #[container]
        #[static_dispatch_for(Ledger as Shelves)]
        #[extend_static_dispatch(Shelves, [$listed])]
        pub struct ShelvedAccounts {
            #[singleton(implementation = Shelf<Paper>)]
            main: Arc<Shelves>,
        }
    };
}

shelved!(Shelf<self::Paper>);

/// What the field answers from, and what a value built by `of` answers from.
pub fn shelved_sources() -> (String, String) {
    let accounts = ShelvedAccounts::new();
    let built = Shelves::of(Shelf::<Paper>::new());
    (accounts.get_main().source(), built.source())
}

/// The same, for a container a function declares.
pub fn sources_in_a_function() -> (String, String) {
    #[container]
    #[static_dispatch_for(Ledger as Local)]
    #[extend_static_dispatch(Local, [self::Shelf<self::Paper>])]
    struct InAFunction {
        #[singleton(implementation = Shelf<Paper>)]
        main: Arc<Local>,
    }

    let accounts = InAFunction::new();
    let built = Local::of(self::Shelf::<Paper>::new());
    (accounts.get_main().source(), built.source())
}
