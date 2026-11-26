mod kept {
    #[snapfit::static_dispatch]
    pub trait Base {
        fn kept(&self) -> &'static str;
    }
}

mod written {
    pub trait Base {
        fn written(&self) -> &'static str;
    }
}

#[snapfit::static_dispatch]
pub trait Database: kept::Base + written::Base {
    fn source(&self) -> &'static str;
}
