#![no_std]
//!  The Pebble game assignment
//!Task Description
//!
//! ```text
//! The games rules are the following:
//!
//!    There are two players: User and Program. The first player is chosen randomly.
//!    The game starts with N pebbles (e.g., N=15).
//!    On the player's turn they must remove from 1 to K pebbles 
//!    (e.g., if K=2, then the player removes 1 or 2 pebbles per turn).
//!    The player who takes last pebble(s) is the winner.
//!
//!The Assignment
//!
//!    Write init() function that:
//!        Receives PebblesInit using the msg::load function;
//!        Checks input data for validness;
//!        Chooses the first player using the exec::random function;
//!        Processes the first turn if the first player is Program.
//!        Fills the GameState structure.
//!
//!    Write the handle() function that:
//!        Receives PebblesAction using msg::load function;
//!        Checks input data for validness;
//!        Processes the User's turn and check whether they win;
//!        Processes the Program turn and check whether it wins;
//!        Send a message to the user with the correspondent PebblesEvent;
//!
//!    Write the state() function that returns the GameState structure using the msg::reply function.
//!
//!Additional Information
//!
//! There are two difficulty levels in the game: DifficultyLevel::Easy and DifficultyLevel::Hard. 
//! Program should choose the pebbles count to be removed randomly at the easy level, 
//! and find the best pebbles count (find a winning strategy) at the hard level.
//!
//!Testing
//!
//!  You are to cover program initialization and all actions by tests using the gtest crate.
//!
//!    Check whether the game initialized correctly.
//!    Check all program strategies (you may split the get_random_u32() function 
//!    into two separated implementations for #[cfg(test)] and #[cfg(not(test))] environments).
//!    Check negative scenarios and invalid input data processing.
//!
//! ```

use gstd::{*};
//use gstd::{msg, prelude::*};
use pebbles_game_io::*;
static mut PEBBLES_GAME: Option<GameState> = None;
static mut COUNTER: i32 = 0;

const DEBUG_ME: bool = false;

/// definition came with assignment.
/// The salt is taken from the incoming message, so this random function is highly dependant on its run-time environment.
pub fn get_random_u32() -> u32 {
    let salt = msg::id();
    let (hash, _num) = exec::random(salt.into()).expect("get_random_u32(): random call failed");
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}

/// There are 2 game modes: easy and hard.
/// In easy mode, for each turn, the Program randomly chooses between 1 and the maximum numbewr of pebbles to remove.
/// In hard mode, the Program tries to be a little more clever as the pebbles are running down. At first, the Program
/// takes as many as they can on each turn. If the pebbles left are within the range of the maximum pebbles that can be taken
/// per turn, the Program takes them all to win. The more complicated parts are trying not to take pebbles that would let the user do
/// the same and win in the next round. Always try and keep the amount of pebbles left more than the maximum number of pebbles that can be taken 
/// in a turn so that you don't let your opponent win. 
pub fn get_pebbles_to_remove(game_state: &mut GameState) -> u32 {
    match game_state.difficulty {
        DifficultyLevel::Easy => (get_random_u32() % (game_state.max_pebbles_per_turn)) + 1,
        DifficultyLevel::Hard => {
              if game_state.pebbles_remaining <= game_state.max_pebbles_per_turn { game_state.pebbles_remaining }
              else if game_state.pebbles_remaining > game_state.max_pebbles_per_turn && 
                      game_state.pebbles_remaining < game_state.pebbles_remaining + game_state.max_pebbles_per_turn
                    { game_state.max_pebbles_per_turn -1 }
              else  { game_state.max_pebbles_per_turn }
        }
    }
}
/// Randomly choose who plays first, the User or the Program.
pub fn init_first_player() -> Player {
    match get_random_u32() % 2 {
        0 => Player::User,
        _ => Player::Program,
    }
}

///  Make sure the DifficultyLevel is OK. Compiler would probably do this.
pub fn check_difficulty_level (init_msg_difficulty: DifficultyLevel) -> bool {
    if init_msg_difficulty !=  DifficultyLevel::Easy && 
       init_msg_difficulty !=  DifficultyLevel::Hard { 
            return false;
    }
    true
}

/// Make sure the pebble counts make sense: no negative number (compiler should proably do this u32 can't be negative.)
/// Make sure the max number of pebble per turn is not greater that the total initial number.
pub fn check_pebbles_input(init_msg_pebbles_count: u32, init_msg_max_pebbles_per_turn: u32) -> bool {
    if init_msg_pebbles_count < 1 || 
       init_msg_max_pebbles_per_turn < 1 || 
       init_msg_max_pebbles_per_turn >= init_msg_pebbles_count {
            return false;
    }
    true
}
/// Set up pebbles game, set the number of pebbles, the maximum number of pebbles that can be removed per turn
/// and the game difficulty.
pub fn restart_game(init_msg_difficulty: DifficultyLevel, init_msg_pebbles_count: u32, init_msg_max_pebbles_per_turn: u32) {
    // Initialization code goes here
        if check_difficulty_level(init_msg_difficulty.clone()) == false {
            panic!("Invalid input data: pebbles_count a2yynd max_pebbles_per_turn must be positive");
        }
	if check_pebbles_input(init_msg_pebbles_count, init_msg_max_pebbles_per_turn) == false {
            panic!("Invalid input data: diffulty level either DifficultyLevel::Easy or DifficultyLevel::Hard");
        }
        let first_player: Player = init_first_player();
        let mut pebbles_game = 
            GameState {
              difficulty: init_msg_difficulty, // difficultyLevel::Easy,
              pebbles_count: init_msg_pebbles_count,
              max_pebbles_per_turn: init_msg_max_pebbles_per_turn,
              pebbles_remaining: init_msg_pebbles_count,
              first_player: first_player.clone(),
              winner: None, //Some(Player::User),          
           };
        if first_player == Player::Program {
            let program_turn = get_pebbles_to_remove(&mut pebbles_game);
            pebbles_game.pebbles_remaining -= program_turn;
        }
        //println!("{:?}", pebbles_game);
        if DEBUG_ME { debug!("init(): {:?}", first_player); }
        if DEBUG_ME { debug!("init(): {:?}", pebbles_game); }
        unsafe { PEBBLES_GAME = Some(pebbles_game) };
}
#[no_mangle]
extern "C" fn init() {
      let init_msg: PebblesInit = msg::load().expect("Unable to load the message");   
      if DEBUG_ME { debug!("init(): {:?}", init_msg);  }
      restart_game(init_msg.difficulty, init_msg.pebbles_count, init_msg.max_pebbles_per_turn);
}
/// Process messages (play the game...)
#[no_mangle]
extern "C" fn handle() {
    let action: PebblesAction = msg::load().expect("Unable to decode `Action`");
    if DEBUG_ME { debug!("handle(): {:?}", action); }
    let mut pebbles_game = unsafe { PEBBLES_GAME.get_or_insert(Default::default()) };
    match action {
        PebblesAction::GiveUp => { // we got a winner and it ain't you
                 pebbles_game.winner = Some(Player::Program);
                 let _result = msg::reply(PebblesEvent::Won(pebbles_game.winner
                                 .as_ref()
                                 .expect("winner")
                                 .clone()), 0); // stop game, communicate results
                 //exec::leave();
        }
        PebblesAction::Restart { difficulty, pebbles_count, max_pebbles_per_turn } => { // bail, no winner, just start again
                 restart_game(difficulty.clone(), pebbles_count, max_pebbles_per_turn);
                 let _result = msg::reply(PebblesInit {difficulty, pebbles_count, max_pebbles_per_turn}, 0);
                 //exec::leave();
        }
        PebblesAction::Turn(mut x) => { 
            // Player::User
            if x > pebbles_game.max_pebbles_per_turn { x = pebbles_game.max_pebbles_per_turn; } 
            if x < pebbles_game.pebbles_remaining {  pebbles_game.pebbles_remaining -= x; }
            else { pebbles_game.pebbles_remaining = 0; }
            if DEBUG_ME { debug!("handle(user count): {:?}", x); }
            if pebbles_game.pebbles_remaining <= 0 { // we got a winner and it's you
                 // stop game, communicate results
                 pebbles_game.winner = Some(Player::User);
                 if DEBUG_ME { debug!("user is the winner"); }
                 let _result = msg::reply(PebblesEvent::Won(pebbles_game.winner
                                 .as_ref()
                                 .expect("winner")
                                 .clone()), 0); // stop game, communicate results
                 exec::leave();
                 //exec::exit(msg::source());
            } else {
                 //msg::reply(PebblesEvent::CounterTurn(pebbles_game.pebbles_remaining, 0));
            }
            if DEBUG_ME { debug!("handle(): {:?}", pebbles_game); }
            // Player::Program
            let program_turn = get_pebbles_to_remove(&mut pebbles_game);
            if DEBUG_ME { debug!("handle(program count): {:?}", program_turn); }
            if program_turn < pebbles_game.pebbles_remaining {  pebbles_game.pebbles_remaining -= program_turn; }
            else { pebbles_game.pebbles_remaining = 0; }
            if pebbles_game.pebbles_remaining <= 0 { // we got a winner and it's not you
                 // stop game, communicate results
                 pebbles_game.winner = Some(Player::Program);
                 if DEBUG_ME { debug!("program is the winner"); }
                 let _result = msg::reply(PebblesEvent::Won(pebbles_game.winner
                                .as_ref()
                                .expect("winner")
                                .clone()), 0); // stop game, communicate results
                 exec::leave();
                 //exec::exit(msg::source());
            } else {
                 if DEBUG_ME { debug!("handle(): CounterTurn pebbles_remaining{:?}", pebbles_game.pebbles_remaining); }
                 let _result = msg::reply(PebblesEvent::CounterTurn(pebbles_game.pebbles_remaining), 0);
            }
            if DEBUG_ME { debug!("handle(): {:?}", pebbles_game); }
        }
    };
    let mut _pebbles_count = unsafe { COUNTER };
}

/// Provide feedback to the client code, via the get_state() function.
#[no_mangle]
extern "C" fn state() {
    let pebbles_game = unsafe { PEBBLES_GAME.take().expect("Error in taking current state") };
    msg::reply(pebbles_game, 0).expect("Failed to reply state");
}
#[cfg(test)]
mod tests {
use pebbles_game_io::*;
use crate::check_pebbles_input;
use crate::check_difficulty_level;
use gstd::{*};

#[test]
  fn test_check_pebbles_input() {
     let res: bool = check_pebbles_input(0, 0);
     assert!(res ==  false);
     let res: bool = check_pebbles_input(15, 16);
     assert!(res ==  false);
     let res: bool = check_pebbles_input(15, 2);
     assert!(res ==  true);
  }
#[test]
  fn test_check_difficulty_level() {
     let res: bool = check_difficulty_level(DifficultyLevel::Easy);
     assert!(res == true);
     let res: bool = check_difficulty_level(DifficultyLevel::Hard);
     assert!(res == true);
  }
}

