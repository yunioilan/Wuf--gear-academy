 #![no_std]
use gstd::{};
use pebbles_game_io::*;

// Global variable to store the game state
static mut PEBBLES_GAME:Option<GameState> = None;


#[no_mangle]
 extern "C" fn init() {
    let init_msg: PebblesInit = msg::load().expect("Failed to load PebblesInit");
  // Validate the initial pebbles count and maximum pebbles per turn
    assert!(
        init_msg.pebbles_count > 0,
        "Initial pebbles count must be greater than 0"
    );
    assert!(
        init_msg.max_pebbles_per_turn > 0,
        "Max pebbles per turn must be greater than 0"
    );
    assert!(
        init_msg.max_pebbles_per_turn <= init_msg.pebbles_count,
        "Max pebbles per turn must be less than or equal to initial pebbles count"
    );

    // Randomly select the first player (user or program)
    let first_player = if get_random_u32() % 2 == 0 {
        Player::User
    } else {
        Player::Program
    };

    // Initialize the game state
    let mut game_state = GameState {
        pebbles_count: init_msg.pebbles_count,
        max_pebbles_per_turn: init_msg.max_pebbles_per_turn,
        pebbles_remaining: init_msg.pebbles_count,
        difficulty: init_msg.difficulty,
        first_player: first_player.clone(),
        winner: None,
    };

     // If the first player is the program, perform the program's first turn
    if first_player == Player::Program {
        game_state.pebbles_remaining -= program_turn(&game_state);
        if game_state.pebbles_remaining == 0 {
            game_state.winner = Some(Player::Program);
            msg::reply(PebblesEvent::Won(Player::Program), 0)
                .expect("Failed to reply with Won event");
        } else {
            msg::reply(PebblesEvent::CounterTurn(game_state.pebbles_remaining), 0)
                .expect("Failed to reply with CounterTurn event");
        }
    }

    unsafe {
        PEBBLES_GAME = Some(game_state);
    }
}

#[no_mangle]
 extern "C" fn handle() {
    let action: PebblesAction = msg::load().expect("Failed to load PebblesAction");
    let mut game_state = unsafe { PEBBLES_GAME.take().expect("Game state not initialized") };
    match action {
        PebblesAction::Turn(pebbles) => {
            assert!(
                pebbles > 0 && pebbles <= game_state.max_pebbles_per_turn,
                "Invalid number of pebbles"
            );

            game_state.pebbles_remaining -= pebbles;

            if game_state.pebbles_remaining == 0 {
                game_state.winner = Some(Player::User);
                msg::reply(PebblesEvent::Won(Player::User), 0)
                    .expect("Failed to reply with Won event");
            } else {
                game_state.pebbles_remaining -= program_turn(&game_state);
                if game_state.pebbles_remaining == 0 {
                    game_state.winner = Some(Player::Program);
                    msg::reply(PebblesEvent::Won(Player::Program), 0)
                        .expect("Failed to reply with Won event");
                } else {
                    msg::reply(PebblesEvent::CounterTurn(game_state.pebbles_remaining), 0)
                        .expect("Failed to reply with CounterTurn event");
                }
            }
        }
        PebblesAction::GiveUp => {
            game_state.winner = Some(Player::Program);
            msg::reply(PebblesEvent::Won(Player::Program), 0)
                .expect("Failed to reply with Won event");
        }
        PebblesAction::Restart {
            difficulty,
            pebbles_count,
            max_pebbles_per_turn,
        } => {
            game_state = GameState {
                pebbles_count,
                max_pebbles_per_turn,
                pebbles_remaining: pebbles_count,
                difficulty,
                first_player: if get_random_u32() % 2 == 0 {
                    Player::User
                } else {
                    Player::Program
                },
                winner: None,
            };

            if game_state.first_player == Player::Program {
                game_state.pebbles_remaining -= program_turn(&game_state);
            }
            msg::reply(PebblesEvent::CounterTurn(game_state.pebbles_remaining), 0)
                .expect("Failed to reply with CounterTurn event");
        }
    }
    unsafe {
        PEBBLES_GAME = Some(game_state);
    }
}

#[no_mangle]
 extern "C" fn state() {
    let game_state = unsafe { PEBBLES_GAME.as_ref().expect("Game state not initialized") };
    msg::reply(game_state, 0).expect("Failed to reply with game state");
}


#[cfg(not(test))]
fn get_random_u32() -> u32 {
    let salt = msg::id();
    let (hash, _num) = exec::random(salt.into()).expect("get_random_u32(): random call failed");
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}

#[cfg(test)]
fn get_random_u32() -> u32 {
    41
}

fn program_turn(game_state: &GameState) -> u32 {
    match game_state.difficulty {
        DifficultyLevel::Easy => (get_random_u32() % game_state.max_pebbles_per_turn) + 1,
        DifficultyLevel::Hard => {
            let target = game_state.max_pebbles_per_turn + 1;
            let remainder = game_state.pebbles_remaining % target;
            if remainder == 0 {
                game_state.max_pebbles_per_turn
            } else {
                remainder
            }
        }
    }
}

