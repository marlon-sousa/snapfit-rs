use super::*;

#[test]
fn a_blank_name_is_refused_with_what_it_names() {
    let error = DependencyName::new(" \t").unwrap_err();

    assert_eq!(error.to_string(), "a dependency name cannot be blank");
}

#[test]
fn a_name_keeps_its_text() {
    let name = FunctionName::new("connect").unwrap();

    assert_eq!(name.as_str(), "connect");
    assert_eq!(name.to_string(), "connect");
}

#[test]
fn every_kind_of_name_refuses_blank_text() {
    assert!(ContainerName::new("").is_err());
    assert!(FunctionName::new("").is_err());
    assert!(TypeName::new(" ").is_err());
}
