use std::sync::Arc;
use snapfit::container;

#[derive(Clone)]
pub struct Config;

pub trait Mailer {}
pub struct Smtp;
impl Mailer for Smtp {}

pub struct Cache;

pub struct Report;
impl Report { pub fn new() -> Self { Self } }

#[container]
pub struct App {
    #[singleton]
    config: Config,

    #[singleton(implementation = Smtp)]
    mailer: Arc<dyn Mailer>,

    #[singleton]
    cache: Arc<Cache>,

    #[singleton(deps = [config])]
    report: Arc<Report>,

    #[singleton]
    spare_cache: Arc<Cache>,
}
