use snapfit::__private::{Either, Only};

#[snapfit::static_dispatch]
pub trait Database {
    fn source(&self) -> &'static str;
}

pub struct MySql;
pub struct Mock;
pub struct Sqlite;

impl Database for MySql {
    fn source(&self) -> &'static str {
        "mysql"
    }
}

impl Database for Mock {
    fn source(&self) -> &'static str {
        "mock"
    }
}

type Db = Either<MySql, Only<Mock>>;

pub fn sqlite() -> Db {
    Db::of(Sqlite)
}
