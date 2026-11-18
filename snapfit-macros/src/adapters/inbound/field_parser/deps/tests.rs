use quote::format_ident;
use syn::parse_quote;

use super::super::super::darling_models::DependencyAttrMeta;
use super::*;
use crate::test_support::labelled;

fn deps(attr: syn::Attribute) -> Deps {
    let meta = DependencyAttrMeta::from_attribute(&attr).unwrap();
    read_deps(&DependencyName::new("exporter").unwrap(), meta.deps).unwrap()
}

#[test]
fn a_label_names_the_setter_its_field_is_passed_to() {
    let deps = deps(parse_quote!(
        #[singleton(deps = [disk = folder_files, config, parent(audit)])]
    ));
    let config = Prerequisite::new(DependencyName::new("config").unwrap());

    assert_eq!(deps.siblings, [labelled("folder_files", "disk"), config]);
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

/// The messages of every problem the field's `deps` has.
fn errors(attr: syn::Attribute) -> Vec<String> {
    let meta = DependencyAttrMeta::from_attribute(&attr).unwrap();
    read_deps(&DependencyName::new("exporter").unwrap(), meta.deps)
        .unwrap_err()
        .iter()
        .map(ToString::to_string)
        .collect()
}

#[test]
fn every_bad_entry_is_reported_in_the_order_written() {
    assert_eq!(
        errors(parse_quote!(
            #[singleton(deps = ["", config + 1, config, a::b = config, parent()])]
        )),
        [
            "field 'exporter': a name in the attribute cannot be blank",
            "field 'exporter': the deps entry 'config + 1' is not the name of a field; write another field's name, such as config",
            "field 'exporter': the deps entry 'a::b = config' is not a label; write the setter, then '=', then the field, each a single name, such as disk = folder_files",
            "field 'exporter': the deps entry 'parent()' names no single dependency; parent(...) is planned for a future version of snapfit and is not implemented yet",
        ]
    );
}

#[test]
fn deps_that_is_not_a_list_says_to_write_one() {
    let not_a_list =
        "is not a list of fields; write the fields in square brackets, such as deps = [config]";

    assert_eq!(
        errors(parse_quote!(#[singleton(deps = config)])),
        [format!("field 'exporter': 'deps = config' {not_a_list}")]
    );
    assert_eq!(
        errors(parse_quote!(#[singleton(deps)])),
        [format!("field 'exporter': 'deps' {not_a_list}")]
    );
}

#[test]
fn deps_in_parentheses_says_to_write_the_same_entries_in_square_brackets() {
    let cases: [(syn::Attribute, &str); 3] = [
        (
            parse_quote!(#[singleton(deps(config, disk = folder_files))]),
            "config, disk = folder_files",
        ),
        (parse_quote!(#[singleton(deps(config + 1))]), "config + 1"),
        (parse_quote!(#[singleton(deps())]), ""),
    ];

    for (attr, entries) in cases {
        assert_eq!(
            errors(attr),
            [format!(
                "field 'exporter': 'deps({entries})' writes its fields in parentheses; write them in square brackets: deps = [{entries}]"
            )]
        );
    }
}
