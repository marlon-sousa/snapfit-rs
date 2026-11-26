use quote::quote;
use syn::parse_quote;

use super::*;
use crate::domain::DispatchArguments;

#[test]
fn the_arguments_are_counted_by_kind() {
    let item: syn::Item = parse_quote!(
        trait Database {}
    );
    let cases = [
        (quote!(), (0, 0, 0)),
        (quote!(remote = other::Store), (1, 0, 0)),
        (quote!(remote), (0, 0, 1)),
        (quote!(inline), (0, 0, 1)),
        (quote!(remote = other::Store<u32>), (1, 0, 0)),
        (quote!(remote = other::Store, inline), (1, 0, 1)),
        (quote!(crate = di), (0, 1, 0)),
        (quote!(crate = ::shop::di,), (0, 1, 0)),
        (quote!(crate = di, crate = di), (0, 2, 0)),
        (quote!(crate = "di"), (0, 0, 1)),
        (quote!(crate = di, inline), (0, 1, 1)),
        (quote!(crate), (0, 0, 1)),
    ];
    for (args, (remote, runtime, other)) in cases {
        let written = args.to_string();
        let arguments = DispatchArguments::new(remote, runtime, other);
        assert_eq!(read(args, item.clone()).arguments, arguments, "{written}");
    }
}

#[test]
fn the_crate_setting_names_the_path_the_impls_reach_snapfit_through() {
    let runtime = |args| {
        trait_of(read(
            args,
            parse_quote!(
                trait Database {}
            ),
        ))
        .syntax
        .runtime
    };

    assert_eq!(runtime(quote!()), parse_quote!(::snapfit));
    assert_eq!(runtime(quote!(crate = di)), parse_quote!(di));
    assert_eq!(
        runtime(quote!(crate = ::shop::di)),
        parse_quote!(::shop::di)
    );
}
