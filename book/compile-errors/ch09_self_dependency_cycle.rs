use std::sync::Arc;
use wireup::container;

pub struct LedgerService;

#[container]
pub struct SelfCyclicContainer {
    #[singleton(deps = [ledger])]
    ledger: Arc<LedgerService>,
}
