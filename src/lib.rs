use async_trait::async_trait;
use futures_core::future::BoxFuture;
use futures_util::future::FutureExt;

#[derive(Clone)]
enum Player {
    One,
    Two,
}

enum Turn {
    Continue,
    GameFinished(GameResult),
}

#[derive(Clone, Debug, PartialEq)]
struct GameResult;

enum Game<'a> {
    Invite {
        invitation: BoxFuture<'a, ()>,
        from: Player,
        to: Player,
    },

    WaitingForTurn {
        turn: BoxFuture<'a, Turn>,
        active: Player,
        idle: Player,
    },

    Finished(GameResult),
}

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

#[async_trait]
trait AsyncGame {
    async fn invite(invite: Invite<'_>) -> AfterInvite<'_>;
    async fn waiting_for_turn(waiting_for_turn: WaitingForTurn<'_>) -> AfterWaitingForTurn<'_>;
}

#[async_trait]
impl<'a> AsyncGame for Game<'a> {
    async fn invite(invite: Invite<'_>) -> AfterInvite<'_> {
        invite.invitation.await;
        let turn = (async { Turn::Continue }).boxed();
        AfterInvite::WaitingForTurn(WaitingForTurn {
            turn,
            active: invite.to,
            idle: invite.from,
        })
    }

    async fn waiting_for_turn(waiting_for_turn: WaitingForTurn<'_>) -> AfterWaitingForTurn<'_> {
        match waiting_for_turn.turn.await {
            Turn::Continue => {
                let turn = (async { Turn::GameFinished(GameResult) }).boxed();
                AfterWaitingForTurn::WaitingForTurn(WaitingForTurn {
                    turn,
                    active: waiting_for_turn.idle,
                    idle: waiting_for_turn.active,
                })
            }
            Turn::GameFinished(result) => AfterWaitingForTurn::Finished(result),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::task::Poll;

    use futures_test::task::panic_context;
    use futures_util::future::FutureExt;
    use futures_util::pin_mut;

    #[test]
    fn it_works() {
        let cx = &mut panic_context();
        let invite = (async {}).boxed();
        let machine = Game::start(invite, Player::One, Player::Two);
        pin_mut!(machine);
        assert_eq!(machine.poll_unpin(cx), Poll::Ready(GameResult));
    }
}
