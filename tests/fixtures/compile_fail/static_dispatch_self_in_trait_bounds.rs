#[snapfit::static_dispatch]
pub trait Store<T>: Sized where T: Into<Self> {
    fn source(&self) -> &'static str;
}

#[snapfit::static_dispatch]
pub trait Inline<T: Into<Self>>: Sized {
    fn source(&self) -> &'static str;
}

#[snapfit::static_dispatch]
pub trait Ordered<T> {
    fn rank(&self) -> u8;
}

#[snapfit::static_dispatch]
pub trait Ranked: Ordered<Self> + Sized {
    fn name(&self) -> u8;
}

macro_rules! archive {
    ($bound:ty) => {
        #[snapfit::static_dispatch]
        pub trait Archive<T>: Sized where T: Into<$bound> {
            fn source(&self) -> &'static str;
        }
    };
}

archive!(Self);

#[snapfit::static_dispatch]
pub trait Cloned where Self: Clone {
    fn source(&self) -> &'static str;
}

#[snapfit::static_dispatch]
pub trait Rows where Self::Row: Clone {
    type Row;
    fn row(&self) -> Self::Row;
}

#[snapfit::static_dispatch]
pub trait Keyed: Ordered<Self::Key> where Self: Clone {
    type Key;
    fn key(&self) -> Self::Key;
}
