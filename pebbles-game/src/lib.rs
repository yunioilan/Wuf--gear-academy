 #![no_std]
use gstd::{};
use pebbles_game_io::*;

static mut PEBBLES_GAME:Option<GameState> = None;


#[no_mangle]
 extern "C" fn init() {
    // 加载并验证初始消息
    let init_msg: PebblesInit = msg::load().expect("Failed to load PebblesInit");

    // 确保初始石子数量大于 0
    assert!(
        init_msg.pebbles_count > 0,
        "Initial pebbles count must be greater than 0"
    );

    // 确保每回合最大可取石子数大于 0
    assert!(
        init_msg.max_pebbles_per_turn > 0,
        "Max pebbles per turn must be greater than 0"
    );

    // 确保每回合最大可取石子数不超过初始石子总数
    assert!(
        init_msg.max_pebbles_per_turn <= init_msg.pebbles_count,
        "Max pebbles per turn must be less than or equal to initial pebbles count"
    );

    // 随机选择第一个玩家（用户或程序）
    let first_player = if get_random_u32() % 2 == 0 {
        Player::User
    } else {
        Player::Program
    };

    // 初始化游戏状态
    let mut game_state = GameState {
        pebbles_count: init_msg.pebbles_count,
        max_pebbles_per_turn: init_msg.max_pebbles_per_turn,
        pebbles_remaining: init_msg.pebbles_count,
        difficulty: init_msg.difficulty,
        first_player: first_player.clone(),
        winner: None,
    };

    // 如果第一个玩家是程序，则进行程序的第一回合操作
    if first_player == Player::Program {
        game_state.pebbles_remaining -= program_turn(&game_state);

        // 如果剩余石子数为 0，程序获胜
        if game_state.pebbles_remaining == 0 {
            game_state.winner = Some(Player::Program);
            msg::reply(PebblesEvent::Won(Player::Program), 0)
                .expect("Failed to reply with Won event");
        } else {
            // 否则回复剩余石子数量给玩家
            msg::reply(PebblesEvent::CounterTurn(game_state.pebbles_remaining), 0)
                .expect("Failed to reply with CounterTurn event");
        }
    }

    // 将初始化后的游戏状态存入全局变量中
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

