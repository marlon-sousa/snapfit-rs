use std::sync::Arc;
use snapfit::container;

#[snapfit::static_dispatch]
pub trait Ledger {
    fn source(&self) -> &'static str;
}

pub struct Archive;
impl Archive { pub fn new() -> Self { Self } }
impl Ledger for Archive { fn source(&self) -> &'static str { "module archive" } }

pub fn main() {
    struct Archive;
    impl Archive { fn new() -> Self { Self } }
    impl Ledger for Archive { fn source(&self) -> &'static str { "local archive" } }

    #[container]
    #[static_dispatch_for(Ledger as Books)]
    #[extend_static_dispatch(Books, [self::Archive])]
    struct Accounts {
        #[singleton(implementation = Archive)]
        main: Arc<Books>,
    }

    let _ = Accounts::new();
}
