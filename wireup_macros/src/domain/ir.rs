#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lifecycle {
    #[default]
    Singleton,
    Transient,
}

impl Lifecycle {
    pub fn is_singleton(&self) -> bool {
        matches!(self, Self::Singleton)
    }

    pub fn is_transient(&self) -> bool {
        matches!(self, Self::Transient)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Singleton => "singleton",
            Self::Transient => "transient",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TraitDispatch {
    #[default]
    None,
    Enum {
        trait_name: String,
        impls: Vec<String>,
    },
    Dyn {
        trait_type: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyIR {
    pub name: String,
    pub type_string: String,
    pub prerequisites: Vec<String>,
    pub lifecycle: Lifecycle,
    pub is_async: bool,
    pub constructor_name: String,
    pub custom_builder: Option<String>,
    pub trait_dispatch: TraitDispatch,
}

impl DependencyIR {
    pub fn new(name: impl Into<String>, type_string: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            type_string: type_string.into(),
            prerequisites: Vec::new(),
            lifecycle: Lifecycle::default(),
            is_async: false,
            constructor_name: "new".to_string(),
            custom_builder: None,
            trait_dispatch: TraitDispatch::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerIR {
    pub name: String,
    pub is_async: bool,
    pub parent_type: Option<String>,
    pub introspection: bool,
    pub dependencies: Vec<DependencyIR>,
}

impl ContainerIR {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            is_async: false,
            parent_type: None,
            introspection: false,
            dependencies: Vec::new(),
        }
    }
}
