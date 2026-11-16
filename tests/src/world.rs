use cucumber::World;

use crate::codegen_containers::{
    DerivedAfterContainer, DerivedBeforeContainer, DerivedContainer, TypedFieldsContainer,
};
use crate::compile_harness::{CompileOutcome, compile_fixture};
use crate::containers::{AppContainer, multi_db};
#[cfg(feature = "unstable")]
use crate::unstable_world::UnstableContainers;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioBlueprint {
    SharedSingletons,
    MultiDatabase,
    TypedFields,
    DerivesBeforeAttribute,
    DerivesAfterAttribute,
    #[cfg(feature = "unstable")]
    AsyncShop,
    #[cfg(feature = "unstable")]
    TransientShop,
    #[cfg(feature = "unstable")]
    BuilderShop,
    #[cfg(feature = "unstable")]
    LayeredShop,
}

#[derive(Default, World)]
pub struct ContainerWorld {
    blueprint: Option<ScenarioBlueprint>,
    app_container: Option<AppContainer>,
    multi_db_container: Option<multi_db::AppContainer>,
    typed_fields_container: Option<TypedFieldsContainer>,
    derived_container: Option<Box<dyn DerivedContainer>>,
    selected_fixture: Option<String>,
    compile_outcome: Option<CompileOutcome>,
    #[cfg(feature = "unstable")]
    unstable: UnstableContainers,
}

impl ContainerWorld {
    pub fn select(&mut self, blueprint: ScenarioBlueprint) {
        self.blueprint = Some(blueprint);
    }

    pub fn instantiate_selected(&mut self) {
        match self
            .blueprint
            .expect("scenario blueprint must be configured in Given step")
        {
            ScenarioBlueprint::SharedSingletons => {
                self.app_container = Some(<AppContainer as snapfit::Container>::new());
            }
            ScenarioBlueprint::MultiDatabase => {
                self.multi_db_container =
                    Some(<multi_db::AppContainer as snapfit::Container>::new());
            }
            ScenarioBlueprint::TypedFields => {
                self.typed_fields_container = Some(TypedFieldsContainer::new());
            }
            ScenarioBlueprint::DerivesBeforeAttribute => {
                self.derived_container = Some(Box::new(DerivedBeforeContainer::new()));
            }
            ScenarioBlueprint::DerivesAfterAttribute => {
                self.derived_container = Some(Box::new(DerivedAfterContainer::new()));
            }
            #[cfg(feature = "unstable")]
            ScenarioBlueprint::AsyncShop => {
                panic!("async containers are initialized with initialize_async_selected()")
            }
            #[cfg(feature = "unstable")]
            ScenarioBlueprint::TransientShop => self.unstable.new_transient_shop(),
            #[cfg(feature = "unstable")]
            ScenarioBlueprint::BuilderShop => self.unstable.new_builder_shop(),
            #[cfg(feature = "unstable")]
            ScenarioBlueprint::LayeredShop => self.unstable.new_layered_shop(),
        }
    }

    #[cfg(feature = "unstable")]
    pub async fn initialize_async_selected(&mut self) {
        self.unstable.init_async_shop().await;
    }

    #[cfg(feature = "unstable")]
    pub fn unstable(&self) -> &UnstableContainers {
        &self.unstable
    }

    pub fn app_container(&self) -> &AppContainer {
        self.app_container
            .as_ref()
            .expect("AppContainer must be instantiated before Then assertions")
    }

    pub fn multi_db_container(&self) -> &multi_db::AppContainer {
        self.multi_db_container
            .as_ref()
            .expect("multi_db::AppContainer must be instantiated before Then assertions")
    }

    pub fn typed_fields_container(&self) -> &TypedFieldsContainer {
        self.typed_fields_container
            .as_ref()
            .expect("TypedFieldsContainer must be instantiated before Then assertions")
    }

    pub fn derived_container(&self) -> &dyn DerivedContainer {
        self.derived_container
            .as_deref()
            .expect("a derived container must be instantiated before Then assertions")
    }

    pub fn select_invalid_fixture(&mut self, fixture_name: String) {
        self.selected_fixture = Some(fixture_name);
    }

    pub fn compile_selected_fixture(&mut self) {
        let name = self
            .selected_fixture
            .as_deref()
            .expect("invalid fixture name must be selected in Given step");
        self.compile_outcome = Some(compile_fixture(name));
    }

    pub fn compile_outcome(&self) -> &CompileOutcome {
        self.compile_outcome
            .as_ref()
            .expect("fixture must be compiled before Then assertions")
    }
}

impl std::fmt::Debug for ContainerWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContainerWorld")
            .field("blueprint", &self.blueprint)
            .field("app_container_initialized", &self.app_container.is_some())
            .field(
                "multi_db_container_initialized",
                &self.multi_db_container.is_some(),
            )
            .field("selected_fixture", &self.selected_fixture)
            .field("compile_outcome", &self.compile_outcome)
            .finish()
    }
}
