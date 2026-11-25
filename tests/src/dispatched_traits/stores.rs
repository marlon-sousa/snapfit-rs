//! A trait with a type parameter and a lifetime of its own.

use snapfit::__private::{Either, Only};

pub type Shelves<'a> = Either<Memory<'a>, Only<Counting>>;

#[snapfit::static_dispatch]
pub trait Store<'a, T>
where
    T: Copy + 'a,
{
    fn put(&mut self, item: &'a T) -> usize;
}

#[derive(Debug, Default)]
pub struct Memory<'a> {
    pub items: Vec<&'a u32>,
}

#[derive(Debug, Default)]
pub struct Counting {
    pub count: usize,
}

impl<'a> Store<'a, u32> for Memory<'a> {
    fn put(&mut self, item: &'a u32) -> usize {
        self.items.push(item);
        self.items.len()
    }
}

impl<'a> Store<'a, u32> for Counting {
    fn put(&mut self, _item: &'a u32) -> usize {
        self.count += 10;
        self.count
    }
}
