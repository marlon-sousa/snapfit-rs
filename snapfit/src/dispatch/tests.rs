use super::{Either, Only};

#[derive(Clone, Debug, PartialEq)]
struct MySql;

#[derive(Clone, Debug, PartialEq)]
struct Postgres;

#[derive(Clone, Debug, PartialEq)]
struct Mock(u32);

type Db = Either<MySql, Either<Postgres, Only<Mock>>>;

#[test]
fn of_puts_each_implementation_in_its_own_variant() {
    assert!(matches!(Db::of(MySql), Either::First(MySql)));
    assert!(matches!(
        Db::of(Postgres),
        Either::Rest(Either::First(Postgres))
    ));
    assert!(matches!(
        Db::of(Mock(7)),
        Either::Rest(Either::Rest(Only(Mock(7))))
    ));
}

#[test]
fn a_list_of_one_holds_its_only_implementation() {
    assert!(matches!(Only::<Mock>::of(Mock(3)), Only(Mock(3))));
}

#[test]
fn a_dispatched_value_clones_and_debugs_as_the_variant_that_holds_its_implementation() {
    let db = Db::of(Mock(7));

    assert_eq!(format!("{:?}", db.clone()), "Rest(Rest(Only(Mock(7))))");
}
