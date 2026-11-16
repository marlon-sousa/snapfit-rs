use derive_more::{AsRef, Deref, Display, From, Into};
use derive_new::new;
use strum::{AsRefStr, EnumIs, IntoStaticStr};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, Display, From, Into, AsRef)]
pub struct DependencyName(String);

impl DependencyName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for DependencyName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, Display, From, Into, AsRef)]
pub struct TypeName(String);

impl TypeName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for TypeName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, Display, From, Into, AsRef)]
pub struct ContainerName(String);

impl ContainerName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ContainerName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(EnumIs, AsRefStr, IntoStaticStr, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[strum(serialize_all = "snake_case")]
pub enum Lifecycle {
    #[default]
    Singleton,
    Transient,
}

impl Lifecycle {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Singleton => "singleton",
            Self::Transient => "transient",
        }
    }
}

#[derive(EnumIs, Debug, Clone, PartialEq, Eq, Default)]
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

#[derive(new, Debug, Clone, PartialEq, Eq)]
pub struct DependencyIR {
    #[new(into)]
    pub name: DependencyName,
    #[new(into)]
    pub type_string: TypeName,
    #[new(default)]
    pub prerequisites: Vec<DependencyName>,
    #[new(default)]
    pub lifecycle: Lifecycle,
    #[new(default)]
    pub is_async: bool,
    #[new(value = "\"new\".to_string()")]
    pub constructor_name: String,
    #[new(default)]
    pub custom_builder: Option<String>,
    #[new(default)]
    pub trait_dispatch: TraitDispatch,
}

#[derive(new, Debug, Clone, PartialEq, Eq)]
pub struct ContainerIR {
    #[new(into)]
    pub name: ContainerName,
    #[new(default)]
    pub is_async: bool,
    #[new(default)]
    pub parent_type: Option<TypeName>,
    #[new(default)]
    pub introspection: bool,
    #[new(default)]
    pub dependencies: Vec<DependencyIR>,
}
