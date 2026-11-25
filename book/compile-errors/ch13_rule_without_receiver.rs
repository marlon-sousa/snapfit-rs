#[snapfit::static_dispatch]
pub trait PriceRule {
    fn standard() -> Self;
    fn price(&self, cents: u64) -> u64;
}

fn main() {}
