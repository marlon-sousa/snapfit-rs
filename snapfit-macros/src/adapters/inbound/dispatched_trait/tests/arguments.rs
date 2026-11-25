use quote::quote;
use syn::parse_quote;

use super::*;

#[test]
fn the_arguments_are_read_as_none_remote_or_other() {
    let item: syn::Item = parse_quote!(
        trait Database {}
    );
    let cases = [
        (quote!(), DispatchArguments::None),
        (quote!(remote = other::Store), DispatchArguments::Remote),
        (quote!(remote), DispatchArguments::Other),
        (quote!(inline), DispatchArguments::Other),
        (
            quote!(remote = other::Store<u32>),
            DispatchArguments::Remote,
        ),
        (
            quote!(remote = other::Store, inline),
            DispatchArguments::Remote,
        ),
        (quote!(crate = di), DispatchArguments::None),
        (quote!(crate = ::shop::di,), DispatchArguments::None),
        (quote!(crate = di, crate = di), DispatchArguments::Other),
        (quote!(crate = "di"), DispatchArguments::Other),
        (quote!(crate = di, inline), DispatchArguments::Other),
        (quote!(crate), DispatchArguments::Other),
    ];
    for (args, arguments) in cases {
        let written = args.to_string();
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
