use cucumber::World;
use wireup_bdd::{AppContainer, CompileOutcome, compile_fixture, multi_db};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioBlueprint {
    SharedSingletons,
    MultiDatabase,
}

#[derive(Default, World)]
pub struct ContainerWorld {
    blueprint: Option<ScenarioBlueprint>,
    app_container: Option<AppContainer>,
    multi_db_container: Option<multi_db::AppContainer>,
    selected_fixture: Option<String>,
    compile_outcome: Option<CompileOutcome>,
}

impl ContainerWorld {
    pub fn select_shared_singletons(&mut self) {
        self.blueprint = Some(ScenarioBlueprint::SharedSingletons);
    }

    pub fn select_multi_database(&mut self) {
        self.blueprint = Some(ScenarioBlueprint::MultiDatabase);
    }

    pub fn instantiate_selected(&mut self) {
        match self
            .blueprint
            .expect("scenario blueprint must be configured in Given step")
        {
            ScenarioBlueprint::SharedSingletons => {
                self.app_container = Some(<AppContainer as wireup::Container>::new());
            }
            ScenarioBlueprint::MultiDatabase => {
                self.multi_db_container =
                    Some(<multi_db::AppContainer as wireup::Container>::new());
            }
        }
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
