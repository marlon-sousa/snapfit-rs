use syn::parse_quote;

use super::*;

// Where each token points is pinned by the scenarios of a fallible field
// whose constructor returns a plain value, which rustc reports once only
// when every token of the path is at the field's fallible word.
#[test]
fn the_runtime_path_keeps_its_text_whatever_its_span() {
    let cases: [syn::Path; 3] = [
        parse_quote!(::snapfit),
        parse_quote!(crate::di),
        parse_quote!(::shop::di),
    ];
    for runtime in cases {
        let written = runtime.to_token_stream().to_string();
        assert_eq!(
            spanned_runtime(&runtime, Span::call_site()).to_string(),
            written
        );
    }
}
