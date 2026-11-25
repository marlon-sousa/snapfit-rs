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

#[test]
fn a_dependency_names_the_setter_of_its_own_name() {
    let field = DependencyName::new("disk").unwrap();

    assert_eq!(field.as_setter().as_str(), "disk");
}

#[test]
fn a_type_without_self_paths_drops_each_self_that_starts_a_path() {
    let spelled = |text: &str| TypeName::new(text).unwrap().without_self_paths();

    assert_eq!(spelled("self::Mock"), "Mock");
    assert_eq!(spelled("Cache<self::Order>"), "Cache<Order>");
    assert_eq!(
        spelled("self::Cache<self::Order, self::Item>"),
        "Cache<Order, Item>"
    );
    assert_eq!(spelled("crate::shop::Mock"), "crate::shop::Mock");
    assert_eq!(spelled("itself::Mock"), "itself::Mock");
    assert_eq!(spelled("Self::Mock"), "Self::Mock");
    assert_eq!(spelled("self::self::Mock"), "self::Mock");
    assert_eq!(spelled("super::self::Mock"), "super::self::Mock");
    assert_eq!(spelled("&self::Mock"), "&Mock");
}
