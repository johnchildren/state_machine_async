mod codegen;
mod derived;

extern crate proc_macro;
use proc_macro::TokenStream;

use darling::ast;
use darling::{FromDeriveInput, FromField, FromVariant};
use heck::SnakeCase;
use petgraph::{Direction, Graph};
use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Ident};

use derived::{State, StateField, StateMachineAsync};

#[proc_macro_derive(StateMachineAsync)]
pub fn derive_state_machine_async(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let sm = StateMachineAsync::from_derive_input(&input).expect("failed to parse input");
    let name = sm.ident;
    let generics = sm.generics;

    let variants = match sm.data {
        ast::Data::Enum(variants) => variants,
        _ => panic!("not an enum"),
    };

    let generic_variants = variants
        .iter()
        .map(|variant| codegen::GenericState::from_state(&generics, &variant));

    let states_names: Vec<&syn::Ident> = variants.iter().map(|variant| &variant.ident).collect();
    let states_names_snake_case: Vec<Ident> = states_names
        .iter()
        .map(|ident| ident.to_string().to_snake_case())
        .map(|snake_case_name| Ident::new(&snake_case_name, Span::call_site()))
        .collect();
    let states_after_names: Vec<Ident> = states_names
        .iter()
        .map(|ident| format_ident!("After{}", ident))
        .filter(|after_name| after_name != "AfterFinished") // horrible hack pls remove
        .collect();

    let mut states_graph = Graph::<Ident, ()>::new();
    let invite_node = states_graph.add_node(format_ident!("Invite"));
    let waiting_for_turn_node = states_graph.add_node(format_ident!("WaitingForTurn"));
    let finished_node = states_graph.add_node(format_ident!("Finished"));
    states_graph.add_edge(invite_node, waiting_for_turn_node, ());
    states_graph.add_edge(waiting_for_turn_node, waiting_for_turn_node, ());
    states_graph.add_edge(waiting_for_turn_node, finished_node, ());

    let invite_node_transitions = states_graph
        .neighbors_directed(invite_node, Direction::Outgoing)
        .map(|x| states_graph.node_weight(x).unwrap().clone())
        .collect::<Vec<Ident>>();

    let waiting_for_turn_node_transitions = states_graph
        .neighbors_directed(waiting_for_turn_node, Direction::Outgoing)
        .map(|x| states_graph.node_weight(x).unwrap().clone())
        .filter(|name| name != "Finished") // horrible hack pls remove
        .collect::<Vec<Ident>>();

    /*
    let states_fields: Vec<syn::punctuated::Iter<syn::Field>> =
        variants.iter().map(|x| x.fields.iter()).collect();
    */
    let derived = quote! {
        impl<'a> #name<'a> {
            async fn start(invitation: BoxFuture<'a, ()>, from: Player, to: Player) -> GameResult {
                let mut state = Game::Invite {
                    invitation,
                    from,
                    to,
                };

                loop {
                    match state {
                        Game::Invite {
                            invitation,
                            from,
                            to,
                        } => {
                            let AfterInvite::WaitingForTurn(WaitingForTurn { turn, active, idle }) =
                                Game::invite(Invite {
                                    invitation,
                                    from,
                                    to,
                                })
                                .await;
                            state = Game::WaitingForTurn { turn, active, idle };
                        }
                        Game::WaitingForTurn { turn, active, idle } => {
                            match Game::waiting_for_turn(WaitingForTurn { turn, active, idle }).await {
                                AfterWaitingForTurn::WaitingForTurn(WaitingForTurn {
                                    turn,
                                    active,
                                    idle,
                                }) => {
                                    state = Game::WaitingForTurn {
                                        turn,
                                        active: idle,
                                        idle: active,
                                    };
                                }
                                AfterWaitingForTurn::Finished(result) => {
                                    state = Game::Finished(result);
                                }
                            }
                        }
                        Game::Finished(result) => return result,
                    }
                }
            }
        }

        #(
        #generic_variants
        )*

        enum AfterInvite<'a> {
            #(
           #invite_node_transitions(#invite_node_transitions<'a>),
            )*
        }

        enum AfterWaitingForTurn<'a> {
            #(
           #waiting_for_turn_node_transitions(#waiting_for_turn_node_transitions<'a>),
            )*
            Finished(GameResult),
        }

        #(
        impl<'a> From<#invite_node_transitions<'a>> for AfterInvite<'a> {
            fn from(x: #invite_node_transitions<'a>) -> AfterInvite<'a> {
                AfterInvite::#invite_node_transitions(x)
            }
        }
        )*

        #(
        impl<'a> From<#waiting_for_turn_node_transitions<'a>> for AfterWaitingForTurn<'a> {
            fn from(x: #waiting_for_turn_node_transitions<'a>) -> AfterWaitingForTurn<'a> {
                AfterWaitingForTurn::#waiting_for_turn_node_transitions(x)
            }
        }
        )*

        impl<'a> From<GameResult> for AfterWaitingForTurn<'a> {
            fn from(x: GameResult) -> AfterWaitingForTurn<'a> {
                AfterWaitingForTurn::Finished(x)
            }
        }

        #[async_trait]
        trait AsyncGame {
            #(
            async fn #states_names_snake_case(#states_names_snake_case: #states_names<'_>) -> #states_after_names<'_>;
            )*
        }
    };

    TokenStream::from(derived)
}
