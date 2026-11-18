//! Adapter: the shapes of expression every attribute reader looks through.

/// The expression as written, outside the invisible group a `macro_rules!`
/// puts around what it passes as `$e:expr`, as darling's own readers do.
pub(in crate::adapters::inbound) fn ungrouped(expr: &syn::Expr) -> &syn::Expr {
    match expr {
        syn::Expr::Group(group) => ungrouped(&group.expr),
        other => other,
    }
}

/// The expression as a single word, such as the function a call names or
/// either side of `start = make`.
pub(in crate::adapters::inbound) fn single_word(expr: &syn::Expr) -> Option<&syn::Ident> {
    match ungrouped(expr) {
        syn::Expr::Path(path) if path.qself.is_none() => path.path.get_ident(),
        _ => None,
    }
}
