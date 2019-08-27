extern crate proc_macro;
use proc_macro::TokenStream;

use heck::SnakeCase;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DeriveInput};

#[proc_macro_derive(StateMachineAsync)]
pub fn derive_state_machine_async(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let name = input.ident;

    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("not an enum"),
    };

    let states_names: Vec<&syn::Ident> = variants.iter().map(|variant| &variant.ident).collect();
    let states_names_snake_case: Vec<Ident> = states_names
        .iter()
        .map(|ident| ident.to_string().to_snake_case())
        .map(|snake_case_name| Ident::new(&snake_case_name, Span::call_site()))
        .collect();
    let states_after_names: Vec<Ident> = states_names
        .iter()
        .map(|ident| format!("After{}", ident.to_string()))
        .filter(|after_name| after_name != "AfterFinished") // horrible hack pls remove
        .map(|after_name| Ident::new(&after_name, Span::call_site()))
        .collect();

    let states_fields: Vec<syn::punctuated::Iter<syn::Field>> =
        variants.iter().map(|x| x.fields.iter()).collect();

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

        struct Invite<'a> {
            invitation: BoxFuture<'a, ()>,
            from: Player,
            to: Player,
        }

        struct WaitingForTurn<'a> {
            turn: BoxFuture<'a, Turn>,
            active: Player,
            idle: Player,
        }

        enum AfterInvite<'a> {
            WaitingForTurn(WaitingForTurn<'a>),
        }

        enum AfterWaitingForTurn<'a> {
            WaitingForTurn(WaitingForTurn<'a>),
            Finished(GameResult),
        }

        impl<'a> From<WaitingForTurn<'a>> for AfterInvite<'a> {
            fn from(x: WaitingForTurn<'a>) -> AfterInvite<'a> {
                AfterInvite::WaitingForTurn(x)
            }
        }

        impl<'a> From<WaitingForTurn<'a>> for AfterWaitingForTurn<'a> {
            fn from(x: WaitingForTurn<'a>) -> AfterWaitingForTurn<'a> {
                AfterWaitingForTurn::WaitingForTurn(x)
            }
        }

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
