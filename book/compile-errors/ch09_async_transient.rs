use wireup::container;

pub struct CartCheckout;

#[container(mode = "async")]
pub struct AsyncTransientContainer {
    #[transient(async)]
    checkout: CartCheckout,
}
