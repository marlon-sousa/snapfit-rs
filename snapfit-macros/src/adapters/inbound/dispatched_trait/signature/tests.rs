use syn::parse_quote;

use super::*;

mod calls;

fn shape(method: syn::TraitItemFn) -> MethodShape {
    method_shape(&method.sig)
}

#[test]
fn each_receiver_is_read_as_its_kind() {
    let cases: [(syn::TraitItemFn, Receiver); 9] = [
        (
            parse_quote!(
                fn f();
            ),
            Receiver::None,
        ),
        (
            parse_quote!(
                fn f(self);
            ),
            Receiver::Value,
        ),
        (
            parse_quote!(
                fn f(mut self);
            ),
            Receiver::Value,
        ),
        (
            parse_quote!(
                fn f(self: Self);
            ),
            Receiver::Value,
        ),
        (
            parse_quote!(
                fn f(&self);
            ),
            Receiver::Reference,
        ),
        (
            parse_quote!(
                fn f(&'a mut self);
            ),
            Receiver::Reference,
        ),
        (
            parse_quote!(
                fn f(self: &mut Self);
            ),
            Receiver::Reference,
        ),
        (
            parse_quote!(
                fn f(self: Box<Self>);
            ),
            Receiver::Other,
        ),
        (
            parse_quote!(
                fn f(self: Pin<&mut Self>);
            ),
            Receiver::Other,
        ),
    ];
    for (method, receiver) in cases {
        let written = quote::quote!(#method).to_string();
        assert_eq!(shape(method).receiver, receiver, "{written}");
    }
}

#[test]
fn self_is_forwardable_only_as_receiver_whole_return_path_or_bound() {
    let cases: [(syn::TraitItemFn, SelfUse); 10] = [
        (
            parse_quote!(
                fn f(&self) -> Self;
            ),
            SelfUse::Forwardable,
        ),
        (
            parse_quote!(
                fn f(&self) -> Vec<Self::Row>;
            ),
            SelfUse::Forwardable,
        ),
        (
            parse_quote!(
                fn f(&self) -> <Self as Database>::Row;
            ),
            SelfUse::Forwardable,
        ),
        (
            parse_quote!(
                fn f(self)
                where
                    Self: Sized;
            ),
            SelfUse::Forwardable,
        ),
        (
            parse_quote!(
                fn f(&self, other: Self);
            ),
            SelfUse::Elsewhere,
        ),
        (
            parse_quote!(
                fn f(&self, other: &Self);
            ),
            SelfUse::Elsewhere,
        ),
        (
            parse_quote!(
                fn f(&self) -> Option<Self>;
            ),
            SelfUse::Elsewhere,
        ),
        (
            parse_quote!(
                fn f<T: Into<Self>>(&self, t: T);
            ),
            SelfUse::Elsewhere,
        ),
        (
            parse_quote!(
                fn f<T>(&self, t: T)
                where
                    T: PartialEq<Self>;
            ),
            SelfUse::Elsewhere,
        ),
        (
            parse_quote!(
                fn f(&self, g: impl Fn(&Self));
            ),
            SelfUse::Elsewhere,
        ),
    ];
    for (method, self_use) in cases {
        let written = quote::quote!(#method).to_string();
        assert_eq!(shape(method).self_use, self_use, "{written}");
    }
}

#[test]
fn a_return_is_read_as_self_as_impl_trait_or_plain() {
    let cases: [(syn::TraitItemFn, ReturnShape); 5] = [
        (
            parse_quote!(
                fn f(&self);
            ),
            ReturnShape::Plain,
        ),
        (
            parse_quote!(
                fn f(&self) -> Self::Row;
            ),
            ReturnShape::Plain,
        ),
        (
            parse_quote!(
                fn f(self) -> Self;
            ),
            ReturnShape::ItSelf,
        ),
        (
            parse_quote!(
                fn f(&self) -> impl Iterator<Item = u32>;
            ),
            ReturnShape::ImplTrait,
        ),
        (
            parse_quote!(
                fn f(&self) -> Box<impl Display>;
            ),
            ReturnShape::ImplTrait,
        ),
    ];
    for (method, returns) in cases {
        let written = quote::quote!(#method).to_string();
        assert_eq!(shape(method).returns, returns, "{written}");
    }
}
