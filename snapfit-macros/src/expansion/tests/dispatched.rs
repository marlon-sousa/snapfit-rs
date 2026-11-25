use quote::quote;

use super::*;

fn accounts(field_key: TokenStream) -> TokenStream {
    quote! {
        #[static_dispatch_for(Ledger as Ledgers)]
        #[extend_static_dispatch(Ledgers, [Fake])]
        #[extend_static_dispatch(Ledgers, [Spare])]
        pub struct Accounts {
            #[singleton(#field_key)]
            main: Arc<Ledgers>,
        }
    }
}

#[test]
fn a_dispatched_type_becomes_an_alias_with_the_container_s_visibility_and_its_checks() {
    let tokens = expand_attribute(
        quote!(),
        accounts(quote!(implementation = Disk)),
        FeatureGate::Stable,
    );
    let code = compact(tokens.clone());

    assert!(compile_errors(&tokens).is_empty());
    assert!(code.contains(
        "#[allow(private_interfaces)]pubtypeLedgers=::snapfit::__private::Either<Disk,::snapfit::__private::Either<Fake,::snapfit::__private::Only<Spare>>>;"
    ));
    assert!(code.contains("__snapfit_implements::<Ledgers>"));
    assert!(code.contains("__snapfit_implements::<Spare>"));
    assert!(code.contains("::std::sync::Arc::new(<Ledgers>::of(<Disk>::new()))"));
    assert!(!code.contains("static_dispatch_for"));
}

#[test]
fn a_refused_container_keeps_its_alias_so_that_its_fields_still_name_a_type() {
    let tokens = expand_attribute(quote!(), accounts(quote!()), FeatureGate::Stable);
    let code = compact(tokens.clone());

    assert_eq!(compile_errors(&tokens).len(), 1);
    assert!(code.contains(
        "pubtypeLedgers=::snapfit::__private::Either<Fake,::snapfit::__private::Only<Spare>>;"
    ));
    assert!(code.contains("pubstructAccounts{main:Arc<Ledgers>,}"));
}

#[test]
fn a_container_whose_fields_cannot_be_read_still_writes_each_alias_with_what_reads() {
    let input = quote! {
        #[static_dispatch_for(Ledger as Ledgers)]
        #[static_dispatch_for(Clock as Clocks)]
        #[static_dispatch_for(Store as Stores, extra = [Memory])]
        pub struct Accounts {
            #[singleton(bogus)]
            main: Arc<Ledgers>,
            #[singleton(implementation = Disk)]
            backup: Arc<Ledgers>,
            #[singleton(build = factory, implementation = [Fake])]
            chosen: Arc<Ledgers>,
            clock: Arc<Clocks>,
        }
    };

    let tokens = expand_attribute(quote!(), input, FeatureGate::Stable);
    let code = compact(tokens.clone());

    assert_eq!(
        compile_errors(&tokens).len(),
        3,
        "{:?}",
        compile_errors(&tokens)
    );
    assert!(code.contains(
        "pubtypeLedgers=::snapfit::__private::Either<Disk,::snapfit::__private::Only<Fake>>;"
    ));
    assert!(code.contains("pubtypeClocks=();"));
    assert!(code.contains("pubtypeStores=();"));
}
