#[snapfit::static_dispatch(crate = snapfit, crate = snapfit)]
pub trait Database {
    fn source(&self) -> &'static str;
}

#[snapfit::static_dispatch(crate = 3)]
pub trait Cache {
    fn source(&self) -> &'static str;
}

#[snapfit::static_dispatch(crate = snapfit::<u32>)]
pub trait Queue {
    fn source(&self) -> &'static str;
}
