#[cfg(test)]
mod tests {
    use super::*;
    use gstd::prelude::*;
    use gtest::{Log, Program, System};
    use pebbles_game_io::*;


    fn create_system_and_user() -> (System, u64) {
        let sys = System::new();
        sys.init_logger();
        let user_id = 1;
        sys.mint_to(user_id, 10000000000000); // 初始化用户余额，至少为存在性存款
        (sys, user_id)
    }

    #[test]
    fn test_init_success() {
        let (sys, user_id) = create_system_and_user();
        let program = Program::current(&sys);

        let init_msg = PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 10,
            max_pebbles_per_turn: 3,
        };

        let res = program.send_bytes(user_id, init_msg.encode());
        println!("{:?}", res);

                let state: GameState = program.read_state(()).expect("Failed to read state");
                assert_eq!(state.pebbles_count, 10);
                assert_eq!(state.max_pebbles_per_turn, 3);
                assert_eq!(state.pebbles_remaining, 7);
                assert!(state.first_player == Player::User || state.first_player == Player::Program);

    }

    //通过修改随机数的返回值来确定测试，如果回的是2，则player先开始，
    //如果回的是1，则program先开始
    #[test]
    fn test_who_turn() {
        let (sys, user_id) = create_system_and_user();

        let program = Program::current(&sys);
        let init_msg = PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 10,
            max_pebbles_per_turn: 3,
        };

        program.send_bytes(1, init_msg.encode());
        let turn_action = PebblesAction::Turn(3);
        let res = program.send_bytes(user_id, turn_action.encode());
        println!("{:?}", res);
        let state: GameState = program.read_state(()).expect("Failed to read state");
        println!("State: {:?}", state);
        assert_eq!(state.first_player, Player::Program);
        //assert_eq!(state.winner,Some(Player::Program));

    }


    //通过改变随机数来调整顺序，测试获胜的判定
    #[test]
    fn test_who_wins() {
        let (sys, user_id) = create_system_and_user();
        let program = Program::current(&sys);
        let init_msg = PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 1,
            max_pebbles_per_turn: 1,
        };

       let res=  program.send_bytes(user_id, init_msg.encode());

        //let turn_action = PebblesAction::Turn(1);
        //let ras = program.send_bytes(user_id, turn_action.encode());
        let state: GameState = program.read_state(()).expect("Failed to read state");
        println!("State: {:?}", state);
        println!("{:?}", res);
        assert_eq!(state.winner,Some(Player::Program));
        //assert_eq!(state.winner,Some(Player::User));
    }

    #[test]
    fn test_restart_game() {
        let (sys, user_id) = create_system_and_user();
        let program = Program::current(&sys);
        let init_msg = PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 10,
            max_pebbles_per_turn: 3,
        };

        program.send_bytes(user_id, init_msg.encode());

        let restart_action = PebblesAction::Restart {
            difficulty: DifficultyLevel::Hard,
            pebbles_count: 20,
            max_pebbles_per_turn: 5,
        };

        program.send_bytes(user_id, restart_action.encode());
        let state: GameState = program.read_state(()).expect("Failed to read state");
        println!("{:?}", state);
        assert_eq!(state.pebbles_count, 20);
    }

    #[test]
    fn test_give_up() {
        let (sys, user_id) = create_system_and_user();
        let program = Program::current(&sys);
        let init_msg = PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 10,
            max_pebbles_per_turn: 3,
        };

        program.send_bytes(user_id, init_msg.encode());

        let give_up_action = PebblesAction::GiveUp;
        let res = program.send_bytes(user_id, give_up_action.encode());
        let state: GameState = program.read_state(()).expect("Failed to read state");
        println!("{:?}", state);
        println!("{:?}", res);
        assert_eq!(state.winner,Some(Player::Program));
    }
}
