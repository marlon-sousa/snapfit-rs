use quote::quote;

use super::*;

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
