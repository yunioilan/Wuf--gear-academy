#![no_std]
use game_session_io::*;
use gstd::{exec, msg};

const TRIES_LIMIT: u8 = 5;

static mut GAME_SESSION_STATE: Option<GameSession> = None;

fn get_game_session_mut() -> &'static mut GameSession {
    unsafe {
        GAME_SESSION_STATE
            .as_mut()
            .expect("PEBBLES_GAME is not initialized")
    }
}
fn get_game_session() -> &'static GameSession {
    unsafe {
        GAME_SESSION_STATE
            .as_ref()
            .expect("PEBBLES_GAME is not initialized")
    }
}

#[no_mangle]
extern "C" fn init() {
    // Receives and stores the Wordle program's address (handled at game_session_io)
    let game_session_init: GameSessionInit =
        msg::load().expect("Unable to decode GameSessionInit");
    game_session_init.assert_valid();
    unsafe {
        GAME_SESSION_STATE = Some(game_session_init.into());
    };
}

#[no_mangle]
extern "C" fn handle() {
    let game_session_action: GameSessionAction =
        msg::load().expect("Unable to decode GameSessionAction");
    let game_session = get_game_session_mut();
    match game_session_action {
        // Action 1
        GameSessionAction::StartGame => {
            let user = msg::source();
            // The program checks if a game already exists for the user;
            let session_info = game_session.sessions.entry(user).or_default();
            match &session_info.session_status {
                SessionStatus::ReplyReceived(wordle_event) => {
                    // A reply is sent to notify the user that the game has beeen successfully started.
                    msg::reply::<GameSessionEvent>(wordle_event.into(), 0)
                        .expect("Error in sending a reply");
                    session_info.session_status = SessionStatus::WaitUserInput;
                }
                SessionStatus::Init
                | SessionStatus::GameOver(..)
                | SessionStatus::WaitWordleStartReply => {
                    // It sends a "StartGame" message to the Wordle program;
                    let send_to_wordle_msg_id = msg::send(
                        game_session.wordle_program_id,
                        WordleAction::StartGame { user },
                        0,
                    )
                    .expect("Error in sending a message");

                    session_info.session_id = msg::id();
                    session_info.original_msg_id = msg::id();
                    session_info.send_to_wordle_msg_id = send_to_wordle_msg_id;
                    session_info.tries = 0;
                    session_info.session_status = SessionStatus::WaitWordleStartReply;
                    // Sends a delayed message with action CheckGameStatus to monitor the game's progress (its logic will be described below);
                    // Specify a delay equal to 200 blocks (10 minutes) for the delayed message.
                    msg::send_delayed(
                        exec::program_id(),
                        GameSessionAction::CheckGameStatus {
                            user,
                            session_id: msg::id(),
                        },
                        0,
                        200,
                    )
                    .expect("Error in send_delayed a message");
                    // Utilizes the exec::wait() or exec::wait_for() function to await a response;
                    exec::wait();
                }
                SessionStatus::WaitUserInput | SessionStatus::WaitWordleCheckWordReply => {
                    panic!("The user is in the game");
                }
            }
        }
        // Action 2
        GameSessionAction::CheckWord { word } => {
            let user = msg::source();
            let session_info = game_session.sessions.entry(user).or_default();
            match &session_info.session_status {
                SessionStatus::ReplyReceived(wordle_event) => {
                    // increments the number of tries
                    session_info.tries += 1;
                    // and checks if the word was guessed.
                    if wordle_event.has_guessed() {
                        // If the word has been guessed, it switches the game status to GameOver(Win).
                        session_info.session_status = SessionStatus::GameOver(GameStatus::Win);
                        msg::reply(GameSessionEvent::GameOver(GameStatus::Win), 0)
                            .expect("Error in sending a reply");
                    } else if session_info.tries == TRIES_LIMIT {
                        // If all attempts are used up and the word is not guessed, it switches the game status to GameOver(Lose).
                        session_info.session_status = SessionStatus::GameOver(GameStatus::Lose);
                        msg::reply(GameSessionEvent::GameOver(GameStatus::Lose), 0)
                            .expect("Error in sending a reply");
                    } else {
                        msg::reply::<GameSessionEvent>(wordle_event.into(), 0)
                            .expect("Error in sending a reply");
                        session_info.session_status = SessionStatus::WaitUserInput;
                    }
                }
                // Ensures that a game exists and is in the correct status;
                SessionStatus::WaitUserInput | SessionStatus::WaitWordleCheckWordReply => {
                    // Validates that the submitted word length is five and is in lowercase;
                    assert!(
                        word.len() == 5 && word.chars().all(|c| c.is_lowercase()),
                        "Invalid word"
                    );
                    // Sends a "CheckWord" message to the Wordle program;
                    let send_to_wordle_msg_id = msg::send(
                        game_session.wordle_program_id,
                        WordleAction::CheckWord { user, word },
                        0,
                    )
                    .expect("Error in sending a message");
                    session_info.original_msg_id = msg::id();
                    session_info.send_to_wordle_msg_id = send_to_wordle_msg_id;
                    session_info.session_status = SessionStatus::WaitWordleCheckWordReply;
                    // Utilizes the exec::wait() or exec::wait_for() function to await a reply;
                    exec::wait();
                }
                SessionStatus::Init
                | SessionStatus::WaitWordleStartReply
                | SessionStatus::GameOver(..) => {
                    panic!("The user is not in the game");
                }
            }
        }
        // Action 3
        GameSessionAction::CheckGameStatus { user, session_id } => {
            if msg::source() == exec::program_id() {
                if let Some(session_info) = game_session.sessions.get_mut(&user) {
                    if session_id == session_info.session_id
                        && !matches!(session_info.session_status, SessionStatus::GameOver(..))
                    {
                        session_info.session_status = SessionStatus::GameOver(GameStatus::Lose);
                        msg::send(user, GameSessionEvent::GameOver(GameStatus::Lose), 0)
                            .expect("Error in sending a reply");
                    }
                }
            }
        }
    }
}

#[no_mangle]
extern "C" fn handle_reply() {
    let reply_to = msg::reply_to().expect("Failed to query reply_to data");
    let wordle_event: WordleEvent = msg::load().expect("Unable to decode WordleEvent");
    let game_session = get_game_session_mut();
    let user = wordle_event.get_user();
    if let Some(session_info) = game_session.sessions.get_mut(user) {
        if reply_to == session_info.send_to_wordle_msg_id && session_info.is_wait_reply_status() {
            session_info.session_status = SessionStatus::ReplyReceived(wordle_event);
            exec::wake(session_info.original_msg_id).expect("Failed to wake message");
        }
    }
}

#[no_mangle]
extern "C" fn state() {
    let game_session = get_game_session();
    msg::reply::<GameSessionState>(game_session.into(), 0)
        .expect("failed to encode or reply from state()");
}
