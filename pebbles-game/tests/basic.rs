use pebbles_game_io::*;
use gstd::{*};
use gtest::{Log, Program, System};
use crate::scale_info::prelude::time::SystemTime;
use crate::scale_info::prelude::time::UNIX_EPOCH;

const ADMIN: u64 = 100;
const MAX_NUMBER_OF_TURNS: u32 = 21; // for loop counter
const MAX_PEBBLES_PER_TURN: u32 = 2;
const PEBBLES_COUNT: u32 = 15;
const DIFFICULTY: DifficultyLevel = DifficultyLevel::Easy;


#[test]
fn success_restart_game() {
    let debug_me: bool = false;
    let system = System::new();

    system.init_logger();
    let game = Program::current(&system);
    let program_id= game.id();
    if debug_me { println!("program id: {:?}", program_id); }
    let game_init_result = game.send(
        ADMIN,
        PebblesInit {
                difficulty: DIFFICULTY,
                pebbles_count: PEBBLES_COUNT,
                max_pebbles_per_turn: MAX_PEBBLES_PER_TURN,
        },
    );
    assert!(!game_init_result.main_failed());
    let state: GameState = game.read_state(b"").unwrap();
    if debug_me { println!("state >>>>>>>>>>>>>>>>>>>>>> {:?}", state); }
    for i in 1..MAX_NUMBER_OF_TURNS {
        if i == 3 { 
            let res = game.send(
              ADMIN,
              PebblesAction::Restart {
                difficulty: DIFFICULTY,
                pebbles_count: PEBBLES_COUNT,
                max_pebbles_per_turn: MAX_PEBBLES_PER_TURN,
              },
            );
            if debug_me { println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX  res = {:?}", res); }
            break; 
        }
        let nanos = (SystemTime::now().duration_since(UNIX_EPOCH).expect("REASON").subsec_nanos()%MAX_PEBBLES_PER_TURN)+1;
        if debug_me { println!("Random number of pebbles that user will remove: {nanos}"); }
        let user_choice = nanos;
        let _res = game.send(ADMIN, PebblesAction::Turn(user_choice));
        //let current_game_state = game.state();
        let state: GameState = game.read_state(b"").unwrap();
        let pebbles_remaining: u32 = state.pebbles_remaining;
        if debug_me { println!("{:?} state >>>>>>>>>>>>>>>>>>>>>> {:?}", i, state); }
        if debug_me { println!("{:?} state >>>>>>>>>>>>>>>>>>>>>> {:?}", i, pebbles_remaining); }
        if pebbles_remaining <= 0 { println!("break break"); break; }
        if debug_me { println!("{:?} user chose {:?} pebbles: ", i, user_choice); }
    }
        let nanos = (SystemTime::now().duration_since(UNIX_EPOCH).expect("REASON").subsec_nanos()%MAX_PEBBLES_PER_TURN)+1;
        let user_choice = nanos;
        let _res = game.send(ADMIN, PebblesAction::Turn(user_choice));
        let _res = game.send(ADMIN, PebblesAction::Turn(user_choice));
    let state: GameState = game.read_state(b"").unwrap();
    let pebbles_remaining: u32 = state.pebbles_remaining;
    let winner: Option<Player> = state.winner.clone();
    //let winner: Option<Player> = state.winner.as_ref().expect("REASON").clone();
    if debug_me { println!("state >>>>>>>>>>>>>>>>>>>>>> {:?}", state); }
    if debug_me { println!("state pebbles_remaining >>>>>>>>>>>>>>>>>>>>>> {:?}", pebbles_remaining); }
    if debug_me { println!("state winner >>>>>>>>>>>>>>>>>>>>>> {:?}", winner); }
    assert_ne!(pebbles_remaining, 0);
    assert_eq!(winner, None);
}
#[test]
fn success_giveup() {
    let debug_me: bool = false;
    let system = System::new();

    system.init_logger();
    let game = Program::current(&system);
    let program_id= game.id();
    if debug_me { println!("program id: {:?}", program_id); }
    let game_init_result = game.send(
        ADMIN,
        PebblesInit {
                difficulty: DIFFICULTY,
                pebbles_count: PEBBLES_COUNT,
                max_pebbles_per_turn: MAX_PEBBLES_PER_TURN,
        },
    );
    assert!(!game_init_result.main_failed());
    let state: GameState = game.read_state(b"").unwrap();
    if debug_me { println!("state >>>>>>>>>>>>>>>>>>>>>> {:?}", state); }
    for i in 1..MAX_NUMBER_OF_TURNS {
        if i == 3 { 
           let res = game.send(ADMIN, PebblesAction::GiveUp); 
           if debug_me { println!("{:?} state GIVEUP >>>>>>>>>>>>>>>>>>>>>> {:?}", i, state); }
           if debug_me { println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX  res = {:?}", res); }
           //assert!(res.contains(&Log::builder().payload(PebblesEvent::Moo)));
           //assert!(res.contains(&Log::builder().payload(PebblesEvent::Won)));
           //assert!(res.contains(&Log::builder().payload(PebblesEvent::Won)));
           let expected_log = Log::builder().dest(ADMIN).payload(PebblesEvent::Won(Player::Program));
           assert!(res.contains(&expected_log));//Make sure the message was sent

           // look in res to see the winner
           break; 
        }
        let nanos = (SystemTime::now().duration_since(UNIX_EPOCH).expect("REASON").subsec_nanos()%MAX_PEBBLES_PER_TURN)+1;
        if debug_me { println!("Random number of pebbles that user will remove: {nanos}"); }
        let user_choice = nanos;
        let _res = game.send(ADMIN, PebblesAction::Turn(user_choice));
        // look in res for winner or current count
        //let current_game_state = game.state();
        let state: GameState = game.read_state(b"").unwrap();
        let pebbles_remaining: u32 = state.pebbles_remaining;
        if debug_me { println!("{:?} state >>>>>>>>>>>>>>>>>>>>>> {:?}", i, state); }
        if debug_me { println!("{:?} state >>>>>>>>>>>>>>>>>>>>>> {:?}", i, pebbles_remaining); }
        if pebbles_remaining <= 0 { println!("break break"); break; }
        if debug_me { println!("{:?} user chose {:?} pebbles: ", i, user_choice); }
    }
    let state: GameState = game.read_state(b"").unwrap();
    let pebbles_remaining: u32 = state.pebbles_remaining;
    let winner: Player = state.winner.as_ref().expect("REASON").clone();
    if debug_me { println!("state >>>>>>>>>>>>>>>>>>>>>> {:?}", state); }
    if debug_me { println!("state pebbles_remaining >>>>>>>>>>>>>>>>>>>>>> {:?}", pebbles_remaining); }
    if debug_me { println!("state winner >>>>>>>>>>>>>>>>>>>>>> {:?}", winner); }
    assert_ne!(pebbles_remaining, 0);
    assert_eq!(winner, Player::Program);
}
#[test]
fn success_run_game() {
    let debug_me: bool = false;
    let system = System::new();

    system.init_logger();
    let game = Program::current(&system);
    let program_id= game.id();
    if debug_me { println!("program id: {:?}", program_id); }
    let game_init_result = game.send(
        ADMIN,
        PebblesInit {
                difficulty: DIFFICULTY,
                pebbles_count: PEBBLES_COUNT,
                max_pebbles_per_turn: MAX_PEBBLES_PER_TURN,
        },
    );
    assert!(!game_init_result.main_failed());
    let state: GameState = game.read_state(b"").unwrap();
    if debug_me { println!("state >>>>>>>>>>>>>>>>>>>>>> {:?}", state); }
    for i in 1..MAX_NUMBER_OF_TURNS {
        let nanos = (SystemTime::now().duration_since(UNIX_EPOCH).expect("REASON").subsec_nanos()%MAX_PEBBLES_PER_TURN)+1;
        if debug_me { println!("Random number of pebbles that user will remove: {nanos}"); }
        let user_choice = nanos;
        let res = game.send(ADMIN, PebblesAction::Turn(user_choice));
        if debug_me { println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX  res = {:?}", res); }
        //let current_game_state = game.state();
        let state: GameState = game.read_state(b"").unwrap();
        let pebbles_remaining: u32 = state.pebbles_remaining;
        if debug_me { println!("{:?} state >>>>>>>>>>>>>>>>>>>>>> {:?}", i, state); }
        if debug_me { println!("{:?} state >>>>>>>>>>>>>>>>>>>>>> {:?}", i, pebbles_remaining); }
        if pebbles_remaining <= 0 { println!("break break"); break; }
        if debug_me { println!("{:?} user chose {:?} pebbles: ", i, user_choice); }
    }
    let state: GameState = game.read_state(b"").unwrap();
    let pebbles_remaining: u32 = state.pebbles_remaining;
    let winner: Player = state.winner.as_ref().expect("REASON").clone();
    if debug_me { println!("state >>>>>>>>>>>>>>>>>>>>>> {:?}", state); }
    if debug_me { println!("state pebbles_remaining >>>>>>>>>>>>>>>>>>>>>> {:?}", pebbles_remaining); }
    if debug_me { println!("state winner >>>>>>>>>>>>>>>>>>>>>> {:?}", winner); }
    assert_eq!(pebbles_remaining, 0);
    assert!(winner == Player::Program || winner == Player::User);
}
