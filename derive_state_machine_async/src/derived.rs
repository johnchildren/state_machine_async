/// derived declaratively defines derive input
/// for use with darling.
use darling::ast;
use darling::{FromDeriveInput, FromField, FromVariant};

#[derive(Debug, FromDeriveInput)]
#[darling(supports(enum_any))]
pub struct StateMachineAsync {
    pub ident: syn::Ident,
    pub generics: syn::Generics,
    pub data: ast::Data<State, ()>,
}

#[derive(Debug, FromVariant)]
pub struct State {
    pub ident: syn::Ident,
    pub fields: ast::Fields<StateField>,
}

#[derive(Debug, FromField)]
pub struct StateField {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,
}
