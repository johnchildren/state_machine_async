use darling::ast;
use darling::{FromDeriveInput, FromField, FromVariant};
use heck::SnakeCase;
use petgraph::{Direction, Graph};
use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Ident};

use crate::derived;

pub struct GenericState {
    ident: syn::Ident,
    generics: syn::Generics,
    fields: Vec<StateField>,
}

impl GenericState {
    pub fn from_state(generics: &syn::Generics, state: &derived::State) -> Self {
        Self {
            ident: state.ident.clone(),
            generics: generics.clone(),
            fields: state
                .fields
                .fields
                .iter()
                .map(|sf| StateField::from_state_field(sf))
                .collect(),
        }
    }
}

impl ToTokens for GenericState {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;
        let generics = &self.generics;
        let fields = &self.fields;
        let state_tokens = quote! {
            struct #ident#generics {
                #(
                #fields
                )*
            }
        };
        tokens.extend(state_tokens);
    }
}

pub struct StateField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

impl StateField {
    fn from_state_field(state_field: &derived::StateField) -> Self {
        Self {
            ident: state_field.ident.clone(),
            ty: state_field.ty.clone(),
        }
    }
}

impl ToTokens for StateField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;
        let ty = &self.ty;
        let state_field_tokens = match ident {
            Some(ident) => quote! { #ident: #ty, },
            None => quote! {},
        };
        tokens.extend(state_field_tokens);
    }
}
