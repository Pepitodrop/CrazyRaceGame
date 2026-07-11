use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DEFAULT_BIND: &str = "0.0.0.0:8080";
const TRACK_SCRIPT: &str = "scripts/generate_track.R";
const GENERATED_TRACK: &str = "/tmp/crazy-race-track.tsv";
const FALLBACK_TRACK: &str = "data/default_track.tsv";
const TRUMPSCRIPT_BIN: &str = "/opt/trumpscript/bin/TRUMP";
const TRUMPSCRIPT_PROGRAM: &str = "announcer/race.tr";
const PIET_PROGRAM: &str = "piet/boost_oracle.ppm";

#[derive(Clone, Debug)]
struct TrackSegment {
    terrain: String,
    speed: i32,
    boost: i32,
    recovery: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RaceAction {
    Accelerate,
    Drift,
    Boost,
}

impl RaceAction {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "accelerate" => Some(Self::Accelerate),
            "drift" => Some(Self::Drift),
            "boost" => Some(Self::Boost),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Accelerate => "Accelerate",
            Self::Drift => "Drift",
            Self::Boost => "Piet Boost",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameMode {
    Online,
    Local,
}

#[derive(Clone, Debug)]
struct Player {
    name: String,
    token: String,
    distance: i32,
    energy: i32,
    submitted: Option<RaceAction>,
}

#[derive(Clone, Debug)]
struct Room {
    code: String,
    mode: GameMode,
    players: Vec<Player>,
    local_token: Option<String>,
    local_turn: usize,
    round: u32,
    winner: Option<usize>,
    last_summary: String,
    announcement: String,
    seed: u64,
}

#[derive(Debug)]
struct AppState {
    rooms: HashMap<String, Room>,
    track: Vec<TrackSegment>,
    piet_boost: i32,
    announcements: Vec<String>,
    entropy: u64,
}

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    query: HashMap<String, String>,
    body: HashMap<String, String>,
}

type RouteResponse = (&'static str, &'static str, String, Vec<(String, String)>);
type RouteError = (&'static str, String);

fn main() {
    if env::args().any(|arg| arg == "--healthcheck") {
        let address = env::var("BIND_ADDRESS").unwrap_or_else(|_| DEFAULT_BIND.to_string());
        let port = address.rsplit(':').next().unwrap_or("8080");
        std::process::exit(if TcpStream::connect(format!("127.0.0.1:{port}")).is_ok() {
            0
        } else {
            1
        });
    }

    let seed = env::var("TRACK_SEED")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(42);

    let track = generate_track(seed).unwrap_or_else(|error| {
        eprintln!("R track generation failed ({error}); loading fallback track.");
        load_track(FALLBACK_TRACK)
            .unwrap_or_else(|fallback_error| panic!("failed to load fallback track: {fallback_error}"))
    });

    let piet_boost = run_piet_oracle(PIET_PROGRAM).unwrap_or_else(|error| {
        eprintln!("Piet oracle failed ({error}); using conservative boost 2.");
        2
    });

    let announcements = load_trumpscript_announcements().unwrap_or_else(|error| {
        eprintln!("TrumpScript announcer failed ({error}); using emergency lines.");
        vec![
            "A tremendous move on the track.".to_string(),
            "The crowd has never seen speed like this.".to_string(),
            "A beautiful race, absolutely beautiful.".to_string(),
        ]
    });

    println!(
        "Loaded {} R track segments, Piet boost {}, and {} TrumpScript announcements.",
        track.len(),
        piet_boost,
        announcements.len()
    );

    let entropy = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(seed)
        ^ seed;

    let state = Arc::new(Mutex::new(AppState {
        rooms: HashMap::new(),
        track,
        piet_boost,
        announcements,
        entropy,
    }));

    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| DEFAULT_BIND.to_string());
    let listener = TcpListener::bind(&bind_address)
        .unwrap_or_else(|error| panic!("cannot bind to {bind_address}: {error}"));
    println!("Crazy Race is listening on http://{bind_address}");

    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                let shared = Arc::clone(&state);
                thread::spawn(move || handle_connection(stream, shared));
            }
            Err(error) => eprintln!("connection error: {error}"),
        }
    }
}

fn handle_connection(mut stream: TcpStream, state: Arc<Mutex<AppState>>) {
    let request = match read_request(&mut stream) {
        Ok(request) => request,
        Err(error) => {
            let _ = send_response(
                &mut stream,
                "400 Bad Request",
                "text/plain; charset=utf-8",
                &error,
                &[],
            );
            return;
        }
    };

    let result = route_request(&request, &state);
    let (status, content_type, body, headers) = match result {
        Ok(response) => response,
        Err((status, message)) => (
            status,
            "text/html; charset=utf-8",
            render_error_page(&message),
            Vec::new(),
        ),
    };
    let _ = send_response(&mut stream, status, content_type, &body, &headers);
}

fn route_request(request: &HttpRequest, state: &Arc<Mutex<AppState>>) -> Result<RouteResponse, RouteError> {
    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/") => Ok((
            "200 OK",
            "text/html; charset=utf-8",
            render_landing_page(),
            Vec::new(),
        )),
        ("GET", "/health") => Ok((
            "200 OK",
            "text/plain; charset=utf-8",
            "ok".to_string(),
            Vec::new(),
        )),
        ("POST", "/create") => create_room(request, state),
        ("POST", "/join") => join_room(request, state),
        ("POST", "/local") => create_local_room(request, state),
        ("GET", "/game") => show_game(request, state),
        ("GET", "/local-game") => show_local_game(request, state),
        ("POST", "/action") => submit_action(request, state),
        ("POST", "/local-action") => submit_local_action(request, state),
        ("POST", "/rematch") => rematch(request, state),
        _ => Err(("404 Not Found", "That route does not exist.".to_string())),
    }
}

fn create_room(request: &HttpRequest, state: &Arc<Mutex<AppState>>) -> Result<RouteResponse, RouteError> {
    let name = clean_name(request.body.get("name"))?;
    let mut app = state.lock().map_err(|_| internal_error())?;
    let code = unique_room_code(&mut app);
    let token = next_token(&mut app);
    let seed = next_random(&mut app);
    let announcement = first_announcement(&app);

    app.rooms.insert(
        code.clone(),
        Room {
            code: code.clone(),
            mode: GameMode::Online,
            players: vec![new_player(name, token.clone())],
            local_token: None,
            local_turn: 0,
            round: 1,
            winner: None,
            last_summary: "Room created. Share the room code with your rival.".to_string(),
            announcement,
            seed,
        },
    );

    Ok(redirect_to_game(&code, &token))
}

fn join_room(request: &HttpRequest, state: &Arc<Mutex<AppState>>) -> Result<RouteResponse, RouteError> {
    let name = clean_name(request.body.get("name"))?;
    let code = required_body(request, "room")?.trim().to_ascii_uppercase();
    let mut app = state.lock().map_err(|_| internal_error())?;
    let token = next_token(&mut app);
    let room = app
        .rooms
        .get_mut(&code)
        .ok_or(("404 Not Found", "Room not found.".to_string()))?;

    if room.mode != GameMode::Online {
        return Err(("409 Conflict", "That room is a local game.".to_string()));
    }
    if room.players.len() >= 2 {
        return Err(("409 Conflict", "That room already has two racers.".to_string()));
    }

    room.players.push(new_player(name, token.clone()));
    room.last_summary = "Both racers are connected. Choose your first move.".to_string();
    Ok(redirect_to_game(&code, &token))
}

fn create_local_room(request: &HttpRequest, state: &Arc<Mutex<AppState>>) -> Result<RouteResponse, RouteError> {
    let player_one = clean_name(request.body.get("player_one"))?;
    let player_two = clean_name(request.body.get("player_two"))?;
    if player_one.eq_ignore_ascii_case(&player_two) {
        return Err((
            "400 Bad Request",
            "Use two different racer names for local play.".to_string(),
        ));
    }

    let mut app = state.lock().map_err(|_| internal_error())?;
    let code = unique_room_code(&mut app);
    let local_token = next_token(&mut app);
    let token_one = next_token(&mut app);
    let token_two = next_token(&mut app);
    let seed = next_random(&mut app);
    let announcement = first_announcement(&app);

    app.rooms.insert(
        code.clone(),
        Room {
            code: code.clone(),
            mode: GameMode::Local,
            players: vec![new_player(player_one, token_one), new_player(player_two, token_two)],
            local_token: Some(local_token.clone()),
            local_turn: 0,
            round: 1,
            winner: None,
            last_summary: "Local race ready. Player 1 chooses first, then pass the device.".to_string(),
            announcement,
            seed,
        },
    );

    Ok(redirect_to_local_game(&code, &local_token))
}

fn new_player(name: String, token: String) -> Player {
    Player {
        name,
        token,
        distance: 0,
        energy: 5,
        submitted: None,
    }
}

fn first_announcement(app: &AppState) -> String {
    app.announcements
        .first()
        .cloned()
        .unwrap_or_else(|| "Race ready.".to_string())
}

fn show_game(request: &HttpRequest, state: &Arc<Mutex<AppState>>) -> Result<RouteResponse, RouteError> {
    let code = required_query(request, "room")?.to_ascii_uppercase();
    let token = required_query(request, "token")?;
    let app = state.lock().map_err(|_| internal_error())?;
    let room = app
        .rooms
        .get(&code)
        .ok_or(("404 Not Found", "Room not found.".to_string()))?;
    if room.mode != GameMode::Online {
        return Err(("409 Conflict", "Open this race through local-play mode.".to_string()));
    }
    let player_index = room
        .players
        .iter()
        .position(|player| player.token == token)
        .ok_or(("403 Forbidden", "Invalid racer token.".to_string()))?;
    Ok((
        "200 OK",
        "text/html; charset=utf-8",
        render_online_game_page(room, player_index, &app.track, app.piet_boost),
        Vec::new(),
    ))
}

fn show_local_game(request: &HttpRequest, state: &Arc<Mutex<AppState>>) -> Result<RouteResponse, RouteError> {
    let code = required_query(request, "room")?.to_ascii_uppercase();
    let token = required_query(request, "token")?;
    let app = state.lock().map_err(|_| internal_error())?;
    let room = app
        .rooms
        .get(&code)
        .ok_or(("404 Not Found", "Local game not found.".to_string()))?;
    ensure_local_access(room, &token)?;
    Ok((
        "200 OK",
        "text/html; charset=utf-8",
        render_local_game_page(room, &token, &app.track, app.piet_boost),
        Vec::new(),
    ))
}

fn submit_action(request: &HttpRequest, state: &Arc<Mutex<AppState>>) -> Result<RouteResponse, RouteError> {
    let code = required_body(request, "room")?.to_ascii_uppercase();
    let token = required_body(request, "token")?;
    let action = parse_action(request)?;

    let mut app = state.lock().map_err(|_| internal_error())?;
    let track = app.track.clone();
    let piet_boost = app.piet_boost;
    let announcements = app.announcements.clone();
    let room = app
        .rooms
        .get_mut(&code)
        .ok_or(("404 Not Found", "Room not found.".to_string()))?;

    if room.mode != GameMode::Online {
        return Err(("409 Conflict", "Use the local-play controls for this race.".to_string()));
    }
    if room.players.len() != 2 {
        return Err(("409 Conflict", "Wait for a second racer.".to_string()));
    }
    if room.winner.is_some() {
        return Err(("409 Conflict", "The race is already finished.".to_string()));
    }

    let player_index = room
        .players
        .iter()
        .position(|player| player.token == token)
        .ok_or(("403 Forbidden", "Invalid racer token.".to_string()))?;
    if room.players[player_index].submitted.is_some() {
        return Err(("409 Conflict", "Your move is already locked in.".to_string()));
    }

    room.players[player_index].submitted = Some(action);
    room.last_summary = format!(
        "{} locked in {}. Waiting for the rival.",
        room.players[player_index].name,
        action.label()
    );
    if room.players.iter().all(|player| player.submitted.is_some()) {
        resolve_round(room, &track, piet_boost, &announcements);
    }
    Ok(redirect_to_game(&code, &token))
}

fn submit_local_action(request: &HttpRequest, state: &Arc<Mutex<AppState>>) -> Result<RouteResponse, RouteError> {
    let code = required_body(request, "room")?.to_ascii_uppercase();
    let token = required_body(request, "token")?;
    let action = parse_action(request)?;

    let mut app = state.lock().map_err(|_| internal_error())?;
    let track = app.track.clone();
    let piet_boost = app.piet_boost;
    let announcements = app.announcements.clone();
    let room = app
        .rooms
        .get_mut(&code)
        .ok_or(("404 Not Found", "Local game not found.".to_string()))?;
    ensure_local_access(room, &token)?;
    if room.winner.is_some() {
        return Err(("409 Conflict", "The race is already finished.".to_string()));
    }

    let turn = room.local_turn.min(1);
    if room.players[turn].submitted.is_some() {
        return Err(("409 Conflict", "That move is already locked in.".to_string()));
    }
    room.players[turn].submitted = Some(action);

    if turn == 0 {
        room.local_turn = 1;
        room.last_summary = format!(
            "{} has chosen. Pass the device to {} — the move stays hidden.",
            room.players[0].name, room.players[1].name
        );
    } else {
        resolve_round(room, &track, piet_boost, &announcements);
        room.local_turn = 0;
    }

    Ok(redirect_to_local_game(&code, &token))
}

fn parse_action(request: &HttpRequest) -> Result<RaceAction, RouteError> {
    RaceAction::parse(&required_body(request, "action")?)
        .ok_or(("400 Bad Request", "Unknown race action.".to_string()))
}

fn rematch(request: &HttpRequest, state: &Arc<Mutex<AppState>>) -> Result<RouteResponse, RouteError> {
    let code = required_body(request, "room")?.to_ascii_uppercase();
    let token = required_body(request, "token")?;
    let mut app = state.lock().map_err(|_| internal_error())?;
    let room = app
        .rooms
        .get_mut(&code)
        .ok_or(("404 Not Found", "Room not found.".to_string()))?;

    let allowed = match room.mode {
        GameMode::Online => room.players.iter().any(|player| player.token == token),
        GameMode::Local => room.local_token.as_deref() == Some(token.as_str()),
    };
    if !allowed {
        return Err(("403 Forbidden", "Invalid racer token.".to_string()));
    }

    for player in &mut room.players {
        player.distance = 0;
        player.energy = 5;
        player.submitted = None;
    }
    room.local_turn = 0;
    room.round = 1;
    room.winner = None;
    room.last_summary = "Rematch started. Choose your move.".to_string();

    Ok(match room.mode {
        GameMode::Online => redirect_to_game(&code, &token),
        GameMode::Local => redirect_to_local_game(&code, &token),
    })
}

fn ensure_local_access(room: &Room, token: &str) -> Result<(), RouteError> {
    if room.mode != GameMode::Local {
        return Err(("409 Conflict", "That room is an online race.".to_string()));
    }
    if room.local_token.as_deref() != Some(token) {
        return Err(("403 Forbidden", "Invalid local-game token.".to_string()));
    }
    Ok(())
}
