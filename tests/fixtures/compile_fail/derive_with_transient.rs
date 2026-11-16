use snapfit::Container;

pub struct UserController;

#[derive(Container)]
pub struct DeriveWithTransientContainer {
    #[transient]
    controller: UserController,
}
