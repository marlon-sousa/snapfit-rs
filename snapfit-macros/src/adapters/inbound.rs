mod darling_models;
mod field_parser;
mod type_inspector;

#[cfg(test)]
mod tests;

use derive_new::new;
use proc_macro2::TokenStream;

use crate::domain::{ContainerIR, ContainerName, DependencyIR, DomainError, TypeName};
use crate::ports::ContainerParserPort;
use darling_models::ContainerAttrMeta;

#[derive(new, Debug, Default, Clone)]
pub(crate) struct SynParserAdapter;

impl SynParserAdapter {
    pub(crate) fn parse_with_args(
        &self,
        args: TokenStream,
        input: &syn::ItemStruct,
    ) -> Result<ContainerIR, DomainError> {
        let meta = Self::extract_container_meta(args, &input.attrs)?;
        Self::build_container_ir(&input.ident, &input.fields, meta)
    }

    pub(crate) fn parse_derive_input(
        &self,
        input: &syn::DeriveInput,
    ) -> Result<ContainerIR, DomainError> {
        let syn::Data::Struct(data_struct) = &input.data else {
            return Err(DomainError::InvalidConfiguration(
                "#[container] can only be applied to a struct".to_string(),
            ));
        };
        let meta = Self::extract_container_meta(TokenStream::new(), &input.attrs)?;
        Self::build_container_ir(&input.ident, &data_struct.fields, meta)
    }

    fn build_container_ir(
        ident: &syn::Ident,
        fields: &syn::Fields,
        meta: ContainerAttrMeta,
    ) -> Result<ContainerIR, DomainError> {
        let name = ContainerName::new(ident.to_string())?;
        let dependencies = Self::parse_struct_fields(fields)?;
        let is_async = Self::resolve_async_mode(meta.mode.as_deref(), &dependencies)?;
        let parent_type = meta.parent.map(|p| TypeName::new(p.0)).transpose()?;

        let mut container = ContainerIR::new(name);
        container.is_async = is_async;
        container.parent_type = parent_type;
        container.introspection = meta.introspection;
        container.dependencies = dependencies;
        Ok(container)
    }

    fn extract_container_meta(
        args: TokenStream,
        attrs: &[syn::Attribute],
    ) -> Result<ContainerAttrMeta, DomainError> {
        let container_attrs: Vec<_> = attrs
            .iter()
            .filter(|attr| attr.path().is_ident("container"))
            .collect();

        if container_attrs.len() > 1 || (!args.is_empty() && !container_attrs.is_empty()) {
            return Err(DomainError::InvalidConfiguration(
                "multiple #[container] attributes are not allowed".to_string(),
            ));
        }

        if !args.is_empty() {
            return ContainerAttrMeta::from_tokens(args);
        }

        container_attrs.into_iter().next().map_or_else(
            || Ok(ContainerAttrMeta::default()),
            ContainerAttrMeta::from_attribute,
        )
    }

    fn parse_struct_fields(fields: &syn::Fields) -> Result<Vec<DependencyIR>, DomainError> {
        match fields {
            syn::Fields::Named(named) => {
                named.named.iter().map(field_parser::parse_field).collect()
            }
            syn::Fields::Unit => Ok(Vec::new()),
            syn::Fields::Unnamed(_) => Err(DomainError::InvalidConfiguration(
                "tuple structs are not supported by #[container]; use named fields".to_string(),
            )),
        }
    }

    fn resolve_async_mode(
        mode: Option<&str>,
        dependencies: &[DependencyIR],
    ) -> Result<bool, DomainError> {
        let has_async_dep = dependencies.iter().any(|dep| dep.is_async);
        match mode {
            None => Ok(has_async_dep),
            Some("async") => Ok(true),
            Some("sync") if has_async_dep => Err(DomainError::InvalidConfiguration(
                "container with mode = \"sync\" cannot contain async dependencies".to_string(),
            )),
            Some("sync") => Ok(false),
            Some(other) => Err(DomainError::InvalidConfiguration(format!(
                "invalid container mode '{other}': expected 'sync' or 'async'"
            ))),
        }
    }
}

impl ContainerParserPort for SynParserAdapter {
    fn parse_container(&self, input: &syn::ItemStruct) -> Result<ContainerIR, DomainError> {
        self.parse_with_args(TokenStream::new(), input)
    }
}
