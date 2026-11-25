#[snapfit::static_dispatch(remote = std::fmt::Display)]
pub trait Display {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}
