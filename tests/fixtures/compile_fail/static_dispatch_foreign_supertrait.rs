use std::fmt;

#[snapfit::static_dispatch]
pub trait Shown: fmt::Display + Send {
    fn shown(&self) -> String;
}
