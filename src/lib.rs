use async_trait::async_trait;
use futures_core::future::BoxFuture;
use futures_util::future::FutureExt;

use derive_state_machine_async::StateMachineAsync;

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

#[derive(StateMachineAsync)]
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

#[async_trait]
impl<'a> AsyncGame for Game<'a> {
    async fn invite(invite: Invite<'_>) -> AfterInvite<'_> {
        invite.invitation.await;

        let turn = (async { Turn::Continue }).boxed();

        WaitingForTurn {
            turn,
            active: invite.to,
            idle: invite.from,
        }
        .into()
    }

    async fn waiting_for_turn(waiting_for_turn: WaitingForTurn<'_>) -> AfterWaitingForTurn<'_> {
        match waiting_for_turn.turn.await {
            Turn::Continue => {
                let turn = (async { Turn::GameFinished(GameResult) }).boxed();

                WaitingForTurn {
                    turn,
                    active: waiting_for_turn.idle,
                    idle: waiting_for_turn.active,
                }
                .into()
            }
            Turn::GameFinished(result) => result.into(),
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
