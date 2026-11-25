#[snapfit::static_dispatch]
pub unsafe trait Database {
    fn source(&self) -> &'static str;
}
