pub trait Named {
    fn name(&self) -> &'static str;
}

#[snapfit::static_dispatch]
pub trait Database: Named {
    fn source(&self) -> &'static str;
}
