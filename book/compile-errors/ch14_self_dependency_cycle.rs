use std::sync::Arc;
use snapfit::container;

pub struct LedgerService;

#[container]
pub struct SelfCyclicContainer {
    #[singleton(deps = [ledger])]
    ledger: Arc<LedgerService>,
}
