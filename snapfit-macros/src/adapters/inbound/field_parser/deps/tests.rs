use quote::format_ident;
use syn::parse_quote;

use super::super::super::darling_models::DependencyAttrMeta;
use super::*;

fn deps(attr: syn::Attribute) -> Deps {
    let meta = DependencyAttrMeta::from_attribute(&attr).unwrap();
    read_deps(&DependencyName::new("exporter").unwrap(), meta.deps.0).unwrap()
}

#[test]
fn a_label_names_the_setter_its_field_is_passed_to() {
    let deps = deps(parse_quote!(
        #[singleton(deps = [disk = folder_files, config, parent(audit)])]
    ));
    let disk = Prerequisite {
        field: DependencyName::new("folder_files").unwrap(),
        label: Some(SetterName::new("disk").unwrap()),
    };
    let config = Prerequisite::new(DependencyName::new("config").unwrap());

    assert_eq!(deps.siblings, [disk, config]);
    assert_eq!(deps.parents, [DependencyName::new("audit").unwrap()]);
    assert_eq!(
        deps.setters,
        [format_ident!("disk"), format_ident!("config")]
    );
}

#[test]
fn a_quoted_sibling_is_passed_to_the_setter_of_its_name() {
    let deps = deps(parse_quote!(#[singleton(deps = ["config"])]));

    assert_eq!(deps.setters, [format_ident!("config")]);
}
