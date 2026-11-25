use syn::parse_quote;

use super::*;

#[test]
fn a_path_of_modules_is_read_as_the_path_to_snapfit() {
    let paths: [syn::Expr; 3] = [
        parse_quote!(di),
        parse_quote!(::shop::di),
        parse_quote!(crate::di),
    ];
    for path in paths {
        let written = path.to_token_stream().to_string();
        assert!(
            matches!(RuntimeMeta::from_expr(&path), Ok(RuntimeMeta::Path(_))),
            "{written}"
        );
    }
}

#[test]
fn a_string_or_anything_but_a_path_of_modules_is_kept_as_written() {
    let cases: [(syn::Expr, RuntimeMeta); 4] = [
        (parse_quote!("di"), RuntimeMeta::Quoted("di".to_owned())),
        (parse_quote!(3), RuntimeMeta::Other("3".to_owned())),
        (
            parse_quote!(di::<u32>),
            RuntimeMeta::Other("di::<u32>".to_owned()),
        ),
        (
            parse_quote!(<di>::x),
            RuntimeMeta::Other("<di>::x".to_owned()),
        ),
    ];
    for (value, meta) in cases {
        assert_eq!(RuntimeMeta::from_expr(&value).ok(), Some(meta));
    }
}
