#[snapfit::static_dispatch]
pub trait Named {
    fn name(&self) -> &'static str;
}

pub trait Loud {
    fn shout(&self) -> &'static str;
}

#[snapfit::static_dispatch]
pub trait Database: Named + Loud {
    fn source(&self) -> &'static str;
}
