use crate::domain::{ContainerIR, DomainError};

pub trait ContainerParserPort {
    fn parse_container(&self, input: &syn::ItemStruct) -> Result<ContainerIR, DomainError>;
}
