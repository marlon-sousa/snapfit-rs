use proc_macro2::{Delimiter, Group};
use quote::{ToTokens, quote};

use super::*;

fn written(path: syn::Path) -> String {
    path.to_token_stream().to_string()
}

#[test]
fn without_a_crate_setting_the_runtime_is_reached_through_snapfit() {
    let cases = [quote!(), quote!(mode = "sync"), quote!(crate = "di")];
    for args in cases {
        let given = args.to_string();
        assert_eq!(
            written(readable_runtime_path(&args)),
            ":: snapfit",
            "{given}"
        );
    }
}

#[test]
fn the_crate_setting_is_read_whatever_else_is_wrong_with_the_arguments() {
    let cases = [
        quote!(crate = di),
        quote!(crate = di,),
        quote!(mode = , crate = di),
        quote!(leaf_build = what ever <, crate = di),
        quote!(crate = di, crate = other),
    ];
    for args in cases {
        let given = args.to_string();
        assert_eq!(written(readable_runtime_path(&args)), "di", "{given}");
    }
}

#[test]
fn the_crate_setting_takes_a_path_from_the_root_or_through_a_macro() {
    let passed = Group::new(Delimiter::None, quote!(::shop::di));
    let cases = [
        (quote!(crate = ::shop::di), ":: shop :: di"),
        (quote!(crate = crate::di), "crate :: di"),
        (quote!(crate = #passed), ":: shop :: di"),
    ];
    for (args, path) in cases {
        let given = args.to_string();
        assert_eq!(written(readable_runtime_path(&args)), path, "{given}");
    }
}

#[test]
fn the_settings_are_split_at_the_commas_between_them() {
    let cases = [
        (quote!(), 0),
        (quote!(crate = di), 1),
        (quote!(crate = di,), 1),
        (quote!(crate = di, remote = other::Store<u32>), 2),
        (quote!(crate = di,,), 2),
        (quote!(,), 1),
    ];
    for (args, count) in cases {
        let given = args.to_string();
        assert_eq!(settings(&args).len(), count, "{given}");
    }
}

#[test]
fn a_crate_setting_that_is_no_path_gives_none() {
    let cases = [
        quote!(crate),
        quote!(crate =),
        quote!(crate = "di"),
        quote!(crate = di::<u32>()),
        quote!(remote = di),
    ];
    for setting in cases {
        let given = setting.to_string();
        assert!(crate_path(&setting).is_none(), "{given}");
    }
}
