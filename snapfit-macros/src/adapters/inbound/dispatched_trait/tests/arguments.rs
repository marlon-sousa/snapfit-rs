use proc_macro2::{Delimiter, Group};
use quote::quote;
use syn::parse_quote;

use super::*;
use crate::domain::{DispatchArguments, DispatchSetting, RuntimeValue};

#[test]
fn each_setting_is_read_in_the_order_written() {
    use DispatchSetting::{Other, Remote, Runtime};
    use RuntimeValue::{NotAPath, Path, Quoted};
    let item: syn::Item = parse_quote!(
        trait Database {}
    );
    // As a `meta` fragment of a `macro_rules!` passes them.
    let passed_remote = Group::new(Delimiter::None, quote!(remote = other::Store));
    let passed_crate = Group::new(Delimiter::None, quote!(crate = "di"));
    let passed_other = Group::new(Delimiter::None, quote!(remote));
    let cases = [
        (quote!(), vec![]),
        (quote!(remote = other::Store), vec![Remote]),
        (quote!(remote), vec![Other]),
        (quote!(inline), vec![Other]),
        (quote!(remote = other::Store<u32>), vec![Remote]),
        (quote!(remote = other::Store, inline), vec![Remote, Other]),
        (quote!(crate = di), vec![Runtime(Path)]),
        (quote!(crate = ::shop::di,), vec![Runtime(Path)]),
        (
            quote!(crate = di, crate = di),
            vec![Runtime(Path), Runtime(Path)],
        ),
        (quote!(crate = "di"), vec![Runtime(Quoted("di".to_owned()))]),
        (quote!(crate = 3), vec![Runtime(NotAPath("3".to_owned()))]),
        (
            quote!(crate = di::<u32>),
            vec![Runtime(NotAPath("di::<u32>".to_owned()))],
        ),
        (quote!(crate = di, inline), vec![Runtime(Path), Other]),
        (quote!(crate), vec![Other]),
        (quote!(crate =), vec![Other]),
        (
            quote!(crate = di,, inline),
            vec![Runtime(Path), Other, Other],
        ),
        (quote!(#passed_remote), vec![Remote]),
        (
            quote!(#passed_crate),
            vec![Runtime(Quoted("di".to_owned()))],
        ),
        (quote!(#passed_other), vec![Other]),
    ];
    for (args, settings) in cases {
        let written = args.to_string();
        let arguments = DispatchArguments::new(settings);
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
    assert_eq!(
        runtime(quote!(crate = "x", remote = a::B, crate = di)),
        parse_quote!(di)
    );
}
