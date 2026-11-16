use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::domain::{ConstructionPlan, InstantiationStep};

#[cfg(test)]
mod tests;

pub(super) fn emit_key_enum(plan: &ConstructionPlan) -> TokenStream {
    let key_enum_ident = format_ident!("{}", plan.key_enum_name);
    let variant_idents: Vec<_> = plan.ordered_steps.iter().map(variant_ident).collect();
    let dep_name_lits: Vec<_> = plan
        .ordered_steps
        .iter()
        .map(|step| step.dep_name.as_str())
        .collect();

    quote! {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        pub enum #key_enum_ident {
            #(#variant_idents,)*
        }

        impl #key_enum_ident {
            pub const fn as_str(&self) -> &'static str {
                match *self {
                    #(Self::#variant_idents => #dep_name_lits,)*
                }
            }
        }

        impl ::std::convert::AsRef<str> for #key_enum_ident {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl ::std::str::FromStr for #key_enum_ident {
            type Err = ::wireup::Error;

            fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                match s {
                    #(#dep_name_lits => ::std::result::Result::Ok(Self::#variant_idents),)*
                    unknown => ::std::result::Result::Err(::wireup::Error::UnknownKey(unknown.to_string())),
                }
            }
        }
    }
}

pub(super) fn variant_ident(step: &InstantiationStep) -> syn::Ident {
    format_ident!("{}", step.dep_name.as_str().to_upper_camel_case())
}
