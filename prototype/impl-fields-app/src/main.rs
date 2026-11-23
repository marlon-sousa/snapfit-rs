use snapfit::container;

pub trait SvcTrait {
    fn answer(&self) -> String;
}

#[derive(Clone, Copy)]
pub struct RealSvc;

impl RealSvc {
    pub fn new() -> Self {
        Self
    }
}

impl SvcTrait for RealSvc {
    fn answer(&self) -> String {
        "real".into()
    }
}

#[derive(Clone, Copy)]
pub struct FakeSvc;

impl FakeSvc {
    pub fn new() -> Self {
        Self
    }
}

impl SvcTrait for FakeSvc {
    fn answer(&self) -> String {
        "fake".into()
    }
}

#[derive(Clone)]
pub struct Controller<R: SvcTrait> {
    svc: R,
}

impl<R: SvcTrait> Controller<R> {
    pub fn new(svc: R) -> Self {
        Self { svc }
    }

    pub fn handle(&self) -> String {
        format!("controller -> {}", self.svc.answer())
    }
}

pub trait ControllerTrait {
    fn handle(&self) -> String;
}

impl<R: SvcTrait> ControllerTrait for Controller<R> {
    fn handle(&self) -> String {
        Controller::handle(self)
    }
}

#[derive(Clone)]
pub struct Api<C: ControllerTrait> {
    controller: C,
}

impl<C: ControllerTrait> Api<C> {
    pub fn new(controller: C) -> Self {
        Self { controller }
    }

    pub fn serve(&self) -> String {
        format!("api -> {}", self.controller.handle())
    }
}

#[container]
pub struct App {
    #[singleton(implementation = RealSvc)]
    svc: impl SvcTrait,

    #[singleton(implementation = Controller<impl SvcTrait>, deps = [svc])]
    controller: impl ControllerTrait,

    #[singleton(deps = [controller])]
    api: Api<impl ControllerTrait>,
}

#[container]
pub struct TestApp {
    #[singleton(implementation = FakeSvc)]
    svc: impl SvcTrait,

    #[singleton(deps = [svc])]
    controller: Controller<impl SvcTrait>,
}

fn main() {
    let app = App::new();
    let controller: Controller<RealSvc> = app.get_controller();
    println!("{}", controller.handle());
    println!("{}", app.get_api().serve());
    println!("{}", TestApp::new().get_controller().handle());
}
