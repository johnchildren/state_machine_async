use darling::usage::{GenericsExt, Purpose, UsesLifetimes, UsesTypeParams};
use quote::{quote, ToTokens};
use syn::{GenericParam, Ident, Lifetime, LifetimeDef, TypeParam};

use crate::derived;

pub struct GenericState {
    ident: syn::Ident,
    fields: Vec<StateField>,
    lifetimes: Vec<Lifetime>,
    ty_params: Vec<Ident>,
}

impl GenericState {
    pub fn from_state_and_generics(
        state: &derived::State,
        generics: &syn::Generics,
    ) -> GenericState {
        Self {
            ident: state.ident.clone(),
            fields: state.fields.fields.iter().map(StateField::from).collect(),
            lifetimes: state
                .fields
                .uses_lifetimes(&Purpose::Declare.into(), &generics.declared_lifetimes())
                .iter()
                .cloned()
                .cloned()
                .collect(),
            ty_params: state
                .fields
                .uses_type_params(&Purpose::Declare.into(), &generics.declared_type_params())
                .iter()
                .cloned()
                .cloned()
                .collect(),
        }
    }
}

impl ToTokens for GenericState {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;
        let fields = &self.fields;

        let lifetimes = self
            .lifetimes
            .iter()
            .cloned()
            .map(LifetimeDef::new)
            .map(GenericParam::from);

        // both type parameters and lifetimes
        let generics_params = self
            .ty_params
            .iter()
            .cloned()
            .map(TypeParam::from)
            .map(GenericParam::from)
            .chain(lifetimes)
            .collect::<syn::punctuated::Punctuated<GenericParam, syn::token::Comma>>();

        let generics = syn::Generics {
            lt_token: None,
            params: generics_params,
            gt_token: None,
            where_clause: None,
        };

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

impl StateField {}

impl From<&derived::StateField> for StateField {
    fn from(state_field: &derived::StateField) -> StateField {
        StateField {
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
