/// derived declaratively defines derive input
/// for use with darling.
use darling::ast;
use darling::{uses_lifetimes, uses_type_params};
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

uses_type_params!(State, fields);
uses_lifetimes!(State, fields);

#[derive(Debug, FromField)]
pub struct StateField {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,
}

uses_type_params!(StateField, ty);
uses_lifetimes!(StateField, ty);
