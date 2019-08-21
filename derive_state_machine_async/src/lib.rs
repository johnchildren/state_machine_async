extern crate proc_macro;
use proc_macro::TokenStream;

use quote::quote;

#[proc_macro_derive(StateMachineAsync)]
pub fn derive_state_machine_async(_item: TokenStream) -> TokenStream {
    let derived = quote! {
        impl<'a> Game<'a> {
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
            async fn invite(invite: Invite<'_>) -> AfterInvite<'_>;
            async fn waiting_for_turn(waiting_for_turn: WaitingForTurn<'_>) -> AfterWaitingForTurn<'_>;
        }
    };

    TokenStream::from(derived)
}
