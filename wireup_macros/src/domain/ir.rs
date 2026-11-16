use derive_new::new;
use strum::{AsRefStr, EnumIs, IntoStaticStr};

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
    pub name: String,
    #[new(into)]
    pub type_string: String,
    #[new(default)]
    pub prerequisites: Vec<String>,
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
    pub name: String,
    #[new(default)]
    pub is_async: bool,
    #[new(default)]
    pub parent_type: Option<String>,
    #[new(default)]
    pub introspection: bool,
    #[new(default)]
    pub dependencies: Vec<DependencyIR>,
}
