//! A trait with every kind of item static dispatch forwards, for which clippy
//! must not lint the impls snapfit writes for its dispatch types.

use std::fmt::Display;

use snapfit::__private::{Either, Only};

#[snapfit::static_dispatch]
pub trait Named: Clone {
    fn name(&self) -> &'static str;
}

/// Used only inside this crate, so `async fn` needs no `Send` bound.
#[allow(async_fn_in_trait)]
#[snapfit::static_dispatch]
pub trait Database<'a, T: Copy + 'a = u32>: Named + Send + Sync {
    type Row: Clone;

    fn find(&self, id: T) -> Option<Self::Row>;

    fn tagged<U: Display>(&self, tag: U) -> String;

    fn shown(&self, value: impl Display, (left, right): (u32, u32), _: u8, mut count: u8) -> String {
        count += 1;
        format!("{}: {value} {left} {right} {count}", self.name())
    }

    fn borrowed(&'a self, item: &'a T) -> &'a T;

    fn store(&mut self, item: T);

    fn into_name(mut self) -> String
    where
        Self: Sized,
    {
        self.store_nothing();
        self.name().to_owned()
    }

    fn store_nothing(&mut self) {}

    fn renamed(self) -> Self
    where
        Self: Sized;

    async fn ping(&self) -> bool;

    /// # Safety
    ///
    /// Always safe; the fixture only checks that the call is forwarded.
    unsafe fn raw(&self) -> u8;

    /// A lifetime bound on `Self` is forwarded.
    fn kept(&self) -> u32
    where
        Self: 'a,
    {
        3
    }

    /// Forwarded without the call warning that it is deprecated.
    #[deprecated(note = "use find")]
    fn legacy(&self) -> u32 {
        0
    }
}

#[derive(Clone)]
pub struct MySql;

#[derive(Clone)]
pub struct Mock;

impl Named for MySql {
    fn name(&self) -> &'static str {
        "mysql"
    }
}

impl Named for Mock {
    fn name(&self) -> &'static str {
        "mock"
    }
}

impl<'a> Database<'a> for MySql {
    type Row = u32;

    fn find(&self, id: u32) -> Option<u32> {
        Some(id)
    }

    fn tagged<U: Display>(&self, tag: U) -> String {
        format!("mysql {tag}")
    }

    fn borrowed(&'a self, item: &'a u32) -> &'a u32 {
        item
    }

    fn store(&mut self, _item: u32) {}

    fn renamed(self) -> Self {
        self
    }

    async fn ping(&self) -> bool {
        true
    }

    unsafe fn raw(&self) -> u8 {
        1
    }
}

impl<'a> Database<'a> for Mock {
    type Row = u32;

    fn find(&self, _id: u32) -> Option<u32> {
        None
    }

    fn tagged<U: Display>(&self, tag: U) -> String {
        format!("mock {tag}")
    }

    fn borrowed(&'a self, item: &'a u32) -> &'a u32 {
        item
    }

    fn store(&mut self, _item: u32) {}

    fn renamed(self) -> Self {
        self
    }

    async fn ping(&self) -> bool {
        false
    }

    unsafe fn raw(&self) -> u8 {
        2
    }
}

pub type Db = Either<MySql, Only<Mock>>;

pub fn mock() -> Db {
    Db::of(Mock)
}
