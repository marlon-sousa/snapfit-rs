use crate::domain::{ContainerIR, DomainError};

pub trait ContainerParserPort {
    fn parse_container(&self, input: &syn::DeriveInput) -> Result<ContainerIR, DomainError>;
}
