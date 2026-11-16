use wireup::Container;

pub struct UserController;

#[derive(Container)]
pub struct DeriveWithTransientContainer {
    #[transient]
    controller: UserController,
}
