use crate::domain::errors::DomainError;
use derive_more::{AsRef, Deref, Display, Into};
use derive_new::new;
use strum::{AsRefStr, EnumIs, IntoStaticStr};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, Display, Into, AsRef)]
pub struct DependencyName(String);

impl DependencyName {
    pub fn new(name: impl Into<String>) -> Result<Self, DomainError> {
        let s = name.into();
        if s.trim().is_empty() {
            return Err(DomainError::InvalidConfiguration(
                "dependency name cannot be empty or blank".to_string(),
            ));
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for DependencyName {
    type Error = DomainError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl TryFrom<&str> for DependencyName {
    type Error = DomainError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, Display, Into, AsRef)]
pub struct TypeName(String);

impl TypeName {
    pub fn new(name: impl Into<String>) -> Result<Self, DomainError> {
        let s = name.into();
        if s.trim().is_empty() {
            return Err(DomainError::InvalidConfiguration(
                "type name cannot be empty or blank".to_string(),
            ));
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for TypeName {
    type Error = DomainError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl TryFrom<&str> for TypeName {
    type Error = DomainError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, Display, Into, AsRef)]
pub struct ContainerName(String);

impl ContainerName {
    pub fn new(name: impl Into<String>) -> Result<Self, DomainError> {
        let s = name.into();
        if s.trim().is_empty() {
            return Err(DomainError::InvalidConfiguration(
                "container name cannot be empty or blank".to_string(),
            ));
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ContainerName {
    type Error = DomainError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl TryFrom<&str> for ContainerName {
    type Error = DomainError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(s)
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
    pub name: DependencyName,
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
