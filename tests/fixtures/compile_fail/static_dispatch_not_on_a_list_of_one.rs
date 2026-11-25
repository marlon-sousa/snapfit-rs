use snapfit::__private::Only;

#[snapfit::static_dispatch]
pub trait Database {
    fn source(&self) -> &'static str;
}

pub struct Mock;
pub struct Sqlite;

impl Database for Mock {
    fn source(&self) -> &'static str {
        "mock"
    }
}

type Db = Only<Mock>;

pub fn sqlite() -> Db {
    Db::of(Sqlite)
}
