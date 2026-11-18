use std::sync::Arc;

use cucumber::{then, when};

use crate::world::ContainerWorld;

#[when(
    "the site whose writer's builder takes memory_files as memory and folder_files as disk is instantiated"
)]
fn when_site_instantiated(world: &mut ContainerWorld) {
    world.built.instantiate_site();
}

#[then(
    "the site writer should write to disk through folder_files and to memory through memory_files"
)]
fn then_site_writer_gets_each_part_by_its_setter(world: &mut ContainerWorld) {
    let writer = world.built.site().get_site_writer();

    assert_eq!(writer.disk.place(), "disk");
    assert_eq!(writer.memory.place(), "memory");
}

#[when(
    "the shelves whose catalog's builder takes write_db and read_db by their own names are instantiated"
)]
fn when_shelves_instantiated(world: &mut ContainerWorld) {
    world.built.instantiate_shelves();
}

#[then("the catalog should read from the container's read_db and write to its write_db")]
fn then_catalog_gets_each_database_by_name(world: &mut ContainerWorld) {
    let shelves = world.built.shelves();
    let catalog = shelves.get_catalog();

    assert!(Arc::ptr_eq(&catalog.read_db, &shelves.get_read_db()));
    assert!(Arc::ptr_eq(&catalog.write_db, &shelves.get_write_db()));
    assert!(!Arc::ptr_eq(&catalog.read_db, &catalog.write_db));
}

#[when(
    "the reports whose report's hand-written builder starts with draft and finishes with publish are instantiated"
)]
fn when_reports_instantiated(world: &mut ContainerWorld) {
    world.built.instantiate_reports();
}

#[then("the report should be published with the container's config")]
fn then_report_published_with_config(world: &mut ContainerWorld) {
    let reports = world.built.reports();
    let report = reports.get_report();

    assert_eq!(report.title, "published");
    assert!(Arc::ptr_eq(&report.config, &reports.get_config()));
}
