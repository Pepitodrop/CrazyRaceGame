use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

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
    players: Vec<Player>,
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

fn main() {
    if env::args().any(|arg| arg == "--healthcheck") {
        let address = env::var("BIND_ADDRESS").unwrap_or_else(|_| DEFAULT_BIND.to_string());
        let port = address.rsplit(':').next().unwrap_or("8080");
        let target = format!("127.0.0.1:{port}");
        std::process::exit(if TcpStream::connect(target).is_ok() { 0 } else { 1 });
    }

    let seed = env::var("TRACK_SEED")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(42);

    let track = generate_track(seed).unwrap_or_else(|error| {
        eprintln!("R track generation failed ({error}); loading fallback track.");
        load_track(FALLBACK_TRACK).unwrap_or_else(|fallback_error| {
            panic!("failed to load fallback track: {fallback_error}")
        })
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
            let _ = send_response(&mut stream, "400 Bad Request", "text/plain; charset=utf-8", &error, &[]);
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

type RouteResponse = (&'static str, &'static str, String, Vec<(String, String)>);

type RouteError = (&'static str, String);

fn route_request(
    request: &HttpRequest,
    state: &Arc<Mutex<AppState>>,
) -> Result<RouteResponse, RouteError> {
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
        ("GET", "/game") => show_game(request, state),
        ("POST", "/action") => submit_action(request, state),
        ("POST", "/rematch") => rematch(request, state),
        _ => Err(("404 Not Found", "That route does not exist.".to_string())),
    }
}

fn create_room(
    request: &HttpRequest,
    state: &Arc<Mutex<AppState>>,
) -> Result<RouteResponse, RouteError> {
    let name = clean_name(request.body.get("name"))?;
    let mut app = state.lock().map_err(|_| internal_error())?;
    let code = unique_room_code(&mut app);
    let token = next_token(&mut app);
    let seed = next_random(&mut app);
    let announcement = app
        .announcements
        .first()
        .cloned()
        .unwrap_or_else(|| "Race ready.".to_string());

    app.rooms.insert(
        code.clone(),
        Room {
            code: code.clone(),
            players: vec![Player {
                name,
                token: token.clone(),
                distance: 0,
                energy: 5,
                submitted: None,
            }],
            round: 1,
            winner: None,
            last_summary: "Room created. Share the room code with your rival.".to_string(),
            announcement,
            seed,
        },
    );

    Ok(redirect_to_game(&code, &token))
}

fn join_room(
    request: &HttpRequest,
    state: &Arc<Mutex<AppState>>,
) -> Result<RouteResponse, RouteError> {
    let name = clean_name(request.body.get("name"))?;
    let code = request
        .body
        .get("room")
        .map(|value| value.trim().to_ascii_uppercase())
        .filter(|value| !value.is_empty())
        .ok_or(("400 Bad Request", "Enter a room code.".to_string()))?;

    let mut app = state.lock().map_err(|_| internal_error())?;
    let token = next_token(&mut app);
    let room = app
        .rooms
        .get_mut(&code)
        .ok_or(("404 Not Found", "Room not found.".to_string()))?;

    if room.players.len() >= 2 {
        return Err(("409 Conflict", "That room already has two racers.".to_string()));
    }

    room.players.push(Player {
        name,
        token: token.clone(),
        distance: 0,
        energy: 5,
        submitted: None,
    });
    room.last_summary = "Both racers are connected. Choose your first move.".to_string();

    Ok(redirect_to_game(&code, &token))
}

fn show_game(
    request: &HttpRequest,
    state: &Arc<Mutex<AppState>>,
) -> Result<RouteResponse, RouteError> {
    let code = required_query(request, "room")?.to_ascii_uppercase();
    let token = required_query(request, "token")?;
    let app = state.lock().map_err(|_| internal_error())?;
    let room = app
        .rooms
        .get(&code)
        .ok_or(("404 Not Found", "Room not found.".to_string()))?;
    let player_index = room
        .players
        .iter()
        .position(|player| player.token == token)
        .ok_or(("403 Forbidden", "Invalid racer token.".to_string()))?;

    let body = render_game_page(room, player_index, &app.track, app.piet_boost);
    Ok(("200 OK", "text/html; charset=utf-8", body, Vec::new()))
}

fn submit_action(
    request: &HttpRequest,
    state: &Arc<Mutex<AppState>>,
) -> Result<RouteResponse, RouteError> {
    let code = required_body(request, "room")?.to_ascii_uppercase();
    let token = required_body(request, "token")?;
    let action = RaceAction::parse(&required_body(request, "action")?)
        .ok_or(("400 Bad Request", "Unknown race action.".to_string()))?;

    let mut app = state.lock().map_err(|_| internal_error())?;
    let track = app.track.clone();
    let piet_boost = app.piet_boost;
    let announcements = app.announcements.clone();
    let room = app
        .rooms
        .get_mut(&code)
        .ok_or(("404 Not Found", "Room not found.".to_string()))?;

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

fn rematch(
    request: &HttpRequest,
    state: &Arc<Mutex<AppState>>,
) -> Result<RouteResponse, RouteError> {
    let code = required_body(request, "room")?.to_ascii_uppercase();
    let token = required_body(request, "token")?;
    let mut app = state.lock().map_err(|_| internal_error())?;
    let room = app
        .rooms
        .get_mut(&code)
        .ok_or(("404 Not Found", "Room not found.".to_string()))?;

    if !room.players.iter().any(|player| player.token == token) {
        return Err(("403 Forbidden", "Invalid racer token.".to_string()));
    }

    for player in &mut room.players {
        player.distance = 0;
        player.energy = 5;
        player.submitted = None;
    }
    room.round = 1;
    room.winner = None;
    room.last_summary = "Rematch started. Choose your move.".to_string();

    Ok(redirect_to_game(&code, &token))
}

fn resolve_round(
    room: &mut Room,
    track: &[TrackSegment],
    piet_boost: i32,
    announcements: &[String],
) {
    let mut summaries = Vec::new();
    let finish_line = track.len() as i32;

    for player in &mut room.players {
        let action = player.submitted.take().unwrap_or(RaceAction::Accelerate);
        let segment_index = player.distance.clamp(0, finish_line.saturating_sub(1)) as usize;
        let segment = &track[segment_index];
        let (movement, energy_delta, note) = movement_for(action, player.energy, segment, piet_boost);
        player.distance += movement;
        player.energy = (player.energy + energy_delta).clamp(0, 10);
        summaries.push(format!(
            "{} used {} on {} and moved {} segment{}{}",
            player.name,
            action.label(),
            segment.terrain,
            movement,
            if movement == 1 { "" } else { "s" },
            note
        ));
    }

    let first_finished = room.players[0].distance >= finish_line;
    let second_finished = room.players[1].distance >= finish_line;
    room.winner = match (first_finished, second_finished) {
        (true, false) => Some(0),
        (false, true) => Some(1),
        (true, true) => {
            if room.players[0].distance > room.players[1].distance {
                Some(0)
            } else if room.players[1].distance > room.players[0].distance {
                Some(1)
            } else {
                Some(((room.seed ^ room.round as u64) & 1) as usize)
            }
        }
        (false, false) => None,
    };

    if let Some(index) = room.winner {
        summaries.push(format!("{} wins the race!", room.players[index].name));
    }

    room.last_summary = format!("Round {}: {}.", room.round, summaries.join(" "));
    if !announcements.is_empty() {
        let index = ((room.seed.wrapping_add(room.round as u64)) % announcements.len() as u64) as usize;
        room.announcement = announcements[index].clone();
    }
    room.round += 1;
}

fn movement_for(
    action: RaceAction,
    energy: i32,
    segment: &TrackSegment,
    piet_boost: i32,
) -> (i32, i32, &'static str) {
    match action {
        RaceAction::Accelerate => ((2 + segment.speed).clamp(1, 6), -1, ""),
        RaceAction::Drift => {
            let terrain_bonus = match segment.terrain.as_str() {
                "curve" => 3,
                "mud" => 1,
                _ => 0,
            };
            (
                (2 + terrain_bonus + segment.speed.max(-1)).clamp(1, 6),
                segment.recovery,
                " and recovered energy",
            )
        }
        RaceAction::Boost if energy >= 3 => (
            (2 + segment.speed + segment.boost + piet_boost).clamp(2, 9),
            -3,
            " with the Piet oracle",
        ),
        RaceAction::Boost => (1, 1, " but lacked energy"),
    }
}

fn generate_track(seed: u64) -> Result<Vec<TrackSegment>, String> {
    let status = Command::new("Rscript")
        .args([TRACK_SCRIPT, &seed.to_string(), GENERATED_TRACK])
        .status()
        .map_err(|error| format!("cannot start Rscript: {error}"))?;
    if !status.success() {
        return Err(format!("Rscript exited with {status}"));
    }
    load_track(GENERATED_TRACK)
}

fn load_track(path: &str) -> Result<Vec<TrackSegment>, String> {
    let content = fs::read_to_string(path).map_err(|error| format!("cannot read {path}: {error}"))?;
    let mut track = Vec::new();
    for (line_number, line) in content.lines().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 5 {
            return Err(format!("invalid track row at line {}", line_number + 1));
        }
        track.push(TrackSegment {
            terrain: fields[1].to_string(),
            speed: fields[2].parse().map_err(|_| format!("invalid speed at line {}", line_number + 1))?,
            boost: fields[3].parse().map_err(|_| format!("invalid boost at line {}", line_number + 1))?,
            recovery: fields[4].parse().map_err(|_| format!("invalid recovery at line {}", line_number + 1))?,
        });
    }
    if track.len() < 8 {
        return Err("track must have at least eight segments".to_string());
    }
    Ok(track)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PietColor {
    Chromatic { hue: u8, lightness: u8 },
    White,
    Black,
}

fn run_piet_oracle(path: &str) -> Result<i32, String> {
    let source = fs::read_to_string(path).map_err(|error| format!("cannot read {path}: {error}"))?;
    run_piet_source(&source)
}

fn run_piet_source(source: &str) -> Result<i32, String> {
    let mut tokens = Vec::new();
    for line in source.lines() {
        let clean = line.split('#').next().unwrap_or("");
        tokens.extend(clean.split_whitespace().map(str::to_string));
    }
    let mut iter = tokens.into_iter();
    if iter.next().as_deref() != Some("P3") {
        return Err("Piet source must be an ASCII PPM (P3)".to_string());
    }
    let width: usize = parse_next(&mut iter, "width")?;
    let height: usize = parse_next(&mut iter, "height")?;
    let max_value: i32 = parse_next(&mut iter, "max value")?;
    if height != 1 || width < 2 || max_value != 255 {
        return Err("oracle expects a one-row, 255-range PPM".to_string());
    }

    let mut colors = Vec::with_capacity(width);
    for _ in 0..width {
        let red: i32 = parse_next(&mut iter, "red")?;
        let green: i32 = parse_next(&mut iter, "green")?;
        let blue: i32 = parse_next(&mut iter, "blue")?;
        colors.push(piet_color(red, green, blue)?);
    }

    let mut stack: Vec<i32> = Vec::new();
    let mut output: Vec<i32> = Vec::new();
    let mut cursor = 0usize;

    while cursor < colors.len() {
        let current = colors[cursor];
        let mut block_end = cursor + 1;
        while block_end < colors.len() && colors[block_end] == current {
            block_end += 1;
        }
        if block_end >= colors.len() {
            break;
        }
        let next = colors[block_end];
        if let (
            PietColor::Chromatic { hue, lightness },
            PietColor::Chromatic {
                hue: next_hue,
                lightness: next_lightness,
            },
        ) = (current, next)
        {
            let hue_change = (next_hue + 6 - hue) % 6;
            let light_change = (next_lightness + 3 - lightness) % 3;
            match (hue_change, light_change) {
                (0, 1) => stack.push((block_end - cursor) as i32),
                (0, 2) => {
                    stack.pop();
                }
                (1, 0) => binary_stack_op(&mut stack, |a, b| a + b),
                (1, 1) => binary_stack_op(&mut stack, |a, b| a - b),
                (1, 2) => binary_stack_op(&mut stack, |a, b| a * b),
                (2, 0) => binary_stack_op(&mut stack, |a, b| if b == 0 { a } else { a / b }),
                (2, 1) => binary_stack_op(&mut stack, |a, b| if b == 0 { a } else { a % b }),
                (2, 2) => {
                    if let Some(value) = stack.pop() {
                        stack.push(if value == 0 { 1 } else { 0 });
                    }
                }
                (4, 0) => {
                    if let Some(value) = stack.last().copied() {
                        stack.push(value);
                    }
                }
                (5, 1) => {
                    if let Some(value) = stack.pop() {
                        output.push(value);
                    }
                }
                _ => {}
            }
        }
        cursor = block_end;
    }

    output
        .first()
        .copied()
        .or_else(|| stack.last().copied())
        .ok_or_else(|| "Piet program produced no number".to_string())
}

fn binary_stack_op<F>(stack: &mut Vec<i32>, operation: F)
where
    F: FnOnce(i32, i32) -> i32,
{
    if stack.len() < 2 {
        return;
    }
    let b = stack.pop().unwrap_or(0);
    let a = stack.pop().unwrap_or(0);
    stack.push(operation(a, b));
}

fn piet_color(red: i32, green: i32, blue: i32) -> Result<PietColor, String> {
    let color = match (red, green, blue) {
        (255, 255, 255) => PietColor::White,
        (0, 0, 0) => PietColor::Black,
        (255, 192, 192) => PietColor::Chromatic { hue: 0, lightness: 0 },
        (255, 0, 0) => PietColor::Chromatic { hue: 0, lightness: 1 },
        (192, 0, 0) => PietColor::Chromatic { hue: 0, lightness: 2 },
        (255, 255, 192) => PietColor::Chromatic { hue: 1, lightness: 0 },
        (255, 255, 0) => PietColor::Chromatic { hue: 1, lightness: 1 },
        (192, 192, 0) => PietColor::Chromatic { hue: 1, lightness: 2 },
        (192, 255, 192) => PietColor::Chromatic { hue: 2, lightness: 0 },
        (0, 255, 0) => PietColor::Chromatic { hue: 2, lightness: 1 },
        (0, 192, 0) => PietColor::Chromatic { hue: 2, lightness: 2 },
        (192, 255, 255) => PietColor::Chromatic { hue: 3, lightness: 0 },
        (0, 255, 255) => PietColor::Chromatic { hue: 3, lightness: 1 },
        (0, 192, 192) => PietColor::Chromatic { hue: 3, lightness: 2 },
        (192, 192, 255) => PietColor::Chromatic { hue: 4, lightness: 0 },
        (0, 0, 255) => PietColor::Chromatic { hue: 4, lightness: 1 },
        (0, 0, 192) => PietColor::Chromatic { hue: 4, lightness: 2 },
        (255, 192, 255) => PietColor::Chromatic { hue: 5, lightness: 0 },
        (255, 0, 255) => PietColor::Chromatic { hue: 5, lightness: 1 },
        (192, 0, 192) => PietColor::Chromatic { hue: 5, lightness: 2 },
        _ => return Err(format!("unsupported Piet color ({red}, {green}, {blue})")),
    };
    Ok(color)
}

fn parse_next<T>(iter: &mut impl Iterator<Item = String>, label: &str) -> Result<T, String>
where
    T: std::str::FromStr,
{
    iter.next()
        .ok_or_else(|| format!("missing {label}"))?
        .parse::<T>()
        .map_err(|_| format!("invalid {label}"))
}

fn load_trumpscript_announcements() -> Result<Vec<String>, String> {
    let output = Command::new(TRUMPSCRIPT_BIN)
        .args(["--shut-up", TRUMPSCRIPT_PROGRAM])
        .output()
        .map_err(|error| format!("cannot launch TrumpScript: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "interpreter exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let lines: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(capitalize)
        .collect();
    if lines.is_empty() {
        return Err("interpreter produced no announcements".to_string());
    }
    Ok(lines)
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn read_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .map_err(|error| error.to_string())?;
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 2048];
    let mut header_end = None;

    while buffer.len() < 65_536 {
        let count = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..count]);
        if let Some(position) = find_bytes(&buffer, b"\r\n\r\n") {
            header_end = Some(position + 4);
            break;
        }
    }

    let header_end = header_end.ok_or_else(|| "Malformed HTTP request.".to_string())?;
    let headers_text = String::from_utf8_lossy(&buffer[..header_end]);
    let mut lines = headers_text.lines();
    let request_line = lines.next().ok_or_else(|| "Missing request line.".to_string())?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let target = parts.next().unwrap_or("/");

    let content_length = lines
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("content-length") {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0);

    while buffer.len() < header_end + content_length {
        let count = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..count]);
    }

    let body_end = (header_end + content_length).min(buffer.len());
    let body_text = String::from_utf8_lossy(&buffer[header_end..body_end]);
    let (path, query_text) = target.split_once('?').unwrap_or((target, ""));

    Ok(HttpRequest {
        method,
        path: path.to_string(),
        query: parse_form(query_text),
        body: parse_form(&body_text),
    })
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

fn parse_form(value: &str) -> HashMap<String, String> {
    value
        .split('&')
        .filter(|pair| !pair.is_empty())
        .filter_map(|pair| {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            Some((url_decode(key), url_decode(value)))
        })
        .collect()
}

fn url_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                output.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                if let (Some(high), Some(low)) = (hex_value(bytes[index + 1]), hex_value(bytes[index + 2])) {
                    output.push(high * 16 + low);
                    index += 3;
                } else {
                    output.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&output).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn send_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &str,
    extra_headers: &[(String, String)],
) -> std::io::Result<()> {
    let mut response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nX-Content-Type-Options: nosniff\r\n",
        body.as_bytes().len()
    );
    for (name, value) in extra_headers {
        response.push_str(&format!("{name}: {value}\r\n"));
    }
    response.push_str("\r\n");
    response.push_str(body);
    stream.write_all(response.as_bytes())
}

fn redirect_to_game(code: &str, token: &str) -> RouteResponse {
    let location = format!("/game?room={}&token={}", url_encode(code), url_encode(token));
    (
        "303 See Other",
        "text/plain; charset=utf-8",
        "Redirecting…".to_string(),
        vec![("Location".to_string(), location)],
    )
}

fn url_encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' => (byte as char).to_string(),
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

fn clean_name(value: Option<&String>) -> Result<String, RouteError> {
    let name = value.map(|name| name.trim()).unwrap_or("");
    if name.is_empty() {
        return Err(("400 Bad Request", "Enter a racer name.".to_string()));
    }
    if name.chars().count() > 24 {
        return Err(("400 Bad Request", "Racer names are limited to 24 characters.".to_string()));
    }
    Ok(name.to_string())
}

fn required_query(request: &HttpRequest, key: &str) -> Result<String, RouteError> {
    request
        .query
        .get(key)
        .cloned()
        .filter(|value| !value.is_empty())
        .ok_or(("400 Bad Request", format!("Missing {key}.")))
}

fn required_body(request: &HttpRequest, key: &str) -> Result<String, RouteError> {
    request
        .body
        .get(key)
        .cloned()
        .filter(|value| !value.is_empty())
        .ok_or(("400 Bad Request", format!("Missing {key}.")))
}

fn internal_error() -> RouteError {
    ("500 Internal Server Error", "Game state is temporarily unavailable.".to_string())
}

fn next_random(app: &mut AppState) -> u64 {
    app.entropy ^= app.entropy << 13;
    app.entropy ^= app.entropy >> 7;
    app.entropy ^= app.entropy << 17;
    app.entropy = app.entropy.wrapping_add(0x9E37_79B9_7F4A_7C15);
    app.entropy
}

fn unique_room_code(app: &mut AppState) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    loop {
        let mut value = next_random(app);
        let code: String = (0..6)
            .map(|_| {
                let character = ALPHABET[(value as usize) % ALPHABET.len()] as char;
                value /= ALPHABET.len() as u64;
                character
            })
            .collect();
        if !app.rooms.contains_key(&code) {
            return code;
        }
    }
}

fn next_token(app: &mut AppState) -> String {
    format!("{:016x}{:016x}", next_random(app), next_random(app))
}

fn render_landing_page() -> String {
    page_shell(
        "Crazy Race",
        "",
        r#"
<section class="hero">
  <p class="eyebrow">R × Rust × TrumpScript × Piet</p>
  <h1>Crazy Race</h1>
  <p>A server-rendered, turn-based 1v1 race. No JavaScript. One Docker container.</p>
</section>
<section class="grid two">
  <article class="card">
    <h2>Create a race</h2>
    <form method="post" action="/create">
      <label>Racer name<input name="name" maxlength="24" required autocomplete="nickname"></label>
      <button type="submit">Create room</button>
    </form>
  </article>
  <article class="card">
    <h2>Join a rival</h2>
    <form method="post" action="/join">
      <label>Room code<input name="room" maxlength="6" required autocapitalize="characters"></label>
      <label>Racer name<input name="name" maxlength="24" required autocomplete="nickname"></label>
      <button type="submit">Join room</button>
    </form>
  </article>
</section>
<section class="card stack-note">
  <h2>Four-language engine</h2>
  <p><strong>R</strong> generates the circuit. <strong>Rust</strong> owns networking and game state. <strong>TrumpScript</strong> runs the announcer through its original Python interpreter. <strong>Piet</strong> paints the boost value.</p>
</section>
"#,
    )
}

fn render_game_page(room: &Room, player_index: usize, track: &[TrackSegment], piet_boost: i32) -> String {
    let waiting_for_player = room.players.len() < 2;
    let waiting_for_move = !waiting_for_player
        && room.winner.is_none()
        && room.players[player_index].submitted.is_some();
    let refresh = if waiting_for_player || waiting_for_move {
        r#"<meta http-equiv="refresh" content="2">"#
    } else {
        ""
    };

    let player = &room.players[player_index];
    let opponent = room.players.get(1 - player_index);
    let finish_line = track.len() as i32;
    let winner_banner = room.winner.map(|winner| {
        if winner == player_index {
            "<div class=\"winner\">You won the race.</div>".to_string()
        } else {
            format!(
                "<div class=\"winner rival\">{} won the race.</div>",
                html_escape(&room.players[winner].name)
            )
        }
    }).unwrap_or_default();

    let opponent_card = opponent.map(|rival| render_racer(rival, finish_line, false)).unwrap_or_else(|| {
        "<article class=\"racer card muted\"><h3>Waiting for rival…</h3><p>Share the room code.</p></article>".to_string()
    });

    let actions = if waiting_for_player {
        "<p class=\"notice\">Waiting for the second racer. This page refreshes automatically.</p>".to_string()
    } else if room.winner.is_some() {
        format!(
            r#"<form method="post" action="/rematch" class="actions">
<input type="hidden" name="room" value="{}">
<input type="hidden" name="token" value="{}">
<button type="submit">Start rematch</button>
</form>"#,
            html_escape(&room.code),
            html_escape(&player.token)
        )
    } else if player.submitted.is_some() {
        "<p class=\"notice\">Move locked. Waiting for your rival; this page refreshes automatically.</p>".to_string()
    } else {
        render_action_form(room, player, piet_boost)
    };

    let track_html: String = track
        .iter()
        .enumerate()
        .map(|(index, segment)| {
            let classes = format!("segment {}", html_escape(&segment.terrain));
            format!(
                "<span class=\"{}\" title=\"{}: speed {:+}, boost +{}, recovery +{}\">{}</span>",
                classes,
                html_escape(&segment.terrain),
                segment.speed,
                segment.boost,
                segment.recovery,
                index + 1
            )
        })
        .collect();

    let content = format!(
        r#"
<section class="topbar">
  <div><p class="eyebrow">Room</p><h1>{code}</h1></div>
  <div class="round">Round {round}</div>
</section>
{winner_banner}
<section class="grid two racers">
  {player_card}
  {opponent_card}
</section>
<section class="card announcer">
  <p class="eyebrow">TrumpScript announcer</p>
  <blockquote>“{announcement}”</blockquote>
  <p>{summary}</p>
</section>
<section class="card">
  <h2>R-generated circuit</h2>
  <div class="track">{track_html}</div>
  <div class="legend"><span>▰ straight</span><span>◒ curve</span><span>≈ mud</span><span>▲ jump</span></div>
</section>
<section class="card">
  <h2>Your move</h2>
  {actions}
</section>
<p class="footer-link"><a href="/">Leave race</a></p>
"#,
        code = html_escape(&room.code),
        round = room.round,
        winner_banner = winner_banner,
        player_card = render_racer(player, finish_line, true),
        opponent_card = opponent_card,
        announcement = html_escape(&room.announcement),
        summary = html_escape(&room.last_summary),
        track_html = track_html,
        actions = actions,
    );

    page_shell("Crazy Race", refresh, &content)
}

fn render_racer(player: &Player, finish_line: i32, is_you: bool) -> String {
    let progress = ((player.distance.min(finish_line) * 100) / finish_line.max(1)).clamp(0, 100);
    format!(
        r#"<article class="racer card">
<p class="eyebrow">{role}</p>
<h3>{name}</h3>
<div class="meter"><span style="width:{progress}%"></span></div>
<p><strong>{distance}/{finish_line}</strong> segments · <strong>{energy}/10</strong> energy</p>
</article>"#,
        role = if is_you { "You" } else { "Rival" },
        name = html_escape(&player.name),
        progress = progress,
        distance = player.distance.min(finish_line),
        finish_line = finish_line,
        energy = player.energy,
    )
}

fn render_action_form(room: &Room, player: &Player, piet_boost: i32) -> String {
    format!(
        r#"<form method="post" action="/action" class="actions">
<input type="hidden" name="room" value="{room}">
<input type="hidden" name="token" value="{token}">
<button name="action" value="accelerate"><strong>Accelerate</strong><small>Fast, costs 1 energy</small></button>
<button name="action" value="drift"><strong>Drift</strong><small>Best in curves, restores energy</small></button>
<button name="action" value="boost"><strong>Piet Boost</strong><small>Painted bonus +{piet}, costs 3 energy</small></button>
</form>"#,
        room = html_escape(&room.code),
        token = html_escape(&player.token),
        piet = piet_boost,
    )
}

fn render_error_page(message: &str) -> String {
    let content = format!(
        "<section class=\"card error\"><p class=\"eyebrow\">Race control</p><h1>Something went wrong</h1><p>{}</p><p><a href=\"/\">Back to the garage</a></p></section>",
        html_escape(message)
    );
    page_shell("Race control", "", &content)
}

fn page_shell(title: &str, head_extra: &str, content: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title}</title>
{head_extra}
<style>
:root {{ color-scheme: dark; font-family: Inter, ui-sans-serif, system-ui, sans-serif; background:#080b12; color:#f5f7fb; }}
* {{ box-sizing:border-box; }}
body {{ margin:0; min-height:100vh; background:radial-gradient(circle at 20% 0%,#23314f 0,transparent 38%),radial-gradient(circle at 100% 20%,#3b1d32 0,transparent 32%),#080b12; }}
main {{ width:min(1000px,calc(100% - 32px)); margin:0 auto; padding:56px 0 80px; }}
h1,h2,h3,p {{ margin-top:0; }} h1 {{ font-size:clamp(2.8rem,9vw,6rem); line-height:.9; letter-spacing:-.055em; }} h2 {{ font-size:1.2rem; }} h3 {{ font-size:1.65rem; margin-bottom:14px; }}
a {{ color:#9fd0ff; }} .hero {{ padding:36px 0 44px; max-width:760px; }} .hero p {{ font-size:1.1rem; color:#b7c0d3; }}
.eyebrow {{ text-transform:uppercase; letter-spacing:.18em; font-size:.72rem; font-weight:800; color:#78d8ff; margin-bottom:10px; }}
.grid {{ display:grid; gap:18px; }} .grid.two {{ grid-template-columns:repeat(2,minmax(0,1fr)); }}
.card {{ background:rgba(16,21,33,.84); border:1px solid rgba(255,255,255,.11); border-radius:22px; padding:24px; box-shadow:0 18px 50px rgba(0,0,0,.28); backdrop-filter:blur(12px); }}
label {{ display:grid; gap:8px; color:#cbd3e2; font-weight:700; margin-bottom:15px; }} input {{ width:100%; border:1px solid #3c465a; background:#0b101a; color:white; border-radius:12px; padding:13px 14px; font:inherit; text-transform:none; }}
button {{ border:0; border-radius:14px; background:linear-gradient(135deg,#62d6ff,#8f7cff); color:#07101c; padding:14px 18px; font:inherit; font-weight:900; cursor:pointer; }} button:hover {{ filter:brightness(1.08); }}
.stack-note {{ margin-top:18px; color:#bdc6d8; }} .topbar {{ display:flex; justify-content:space-between; align-items:end; margin-bottom:24px; }} .topbar h1 {{ font-size:clamp(3rem,10vw,5rem); margin:0; }} .round {{ font-weight:900; color:#aeb8ca; padding-bottom:7px; }}
.racers {{ margin-bottom:18px; }} .racer {{ min-height:180px; }} .muted {{ opacity:.62; }} .meter {{ height:13px; border-radius:999px; overflow:hidden; background:#070a10; border:1px solid #333c4e; margin:22px 0 14px; }} .meter span {{ display:block; height:100%; background:linear-gradient(90deg,#4fe0b6,#77b7ff,#aa79ff); }}
.announcer {{ margin-bottom:18px; border-color:rgba(255,211,102,.3); }} blockquote {{ margin:0 0 16px; font-size:clamp(1.3rem,4vw,2rem); font-weight:850; line-height:1.15; color:#ffe59b; }} .announcer > p:last-child {{ color:#b8c0d0; margin-bottom:0; }}
.track {{ display:grid; grid-template-columns:repeat(10,1fr); gap:7px; margin-top:20px; }} .segment {{ aspect-ratio:1; display:grid; place-items:center; border-radius:8px; font-size:.7rem; font-weight:900; background:#344159; }} .segment.curve {{ background:#7552aa; }} .segment.mud {{ background:#76563d; }} .segment.jump {{ background:#c06455; }} .legend {{ display:flex; flex-wrap:wrap; gap:14px; margin-top:15px; color:#9ca7ba; font-size:.85rem; }}
.actions {{ display:grid; grid-template-columns:repeat(3,minmax(0,1fr)); gap:12px; }} .actions button {{ min-height:88px; display:flex; flex-direction:column; justify-content:center; gap:5px; }} .actions small {{ font-weight:650; opacity:.72; }} .notice {{ color:#d8ddeb; margin:0; }} .winner {{ margin-bottom:18px; padding:18px 22px; border-radius:18px; background:#164d3c; color:#a8ffe0; font-size:1.2rem; font-weight:900; }} .winner.rival {{ background:#542638; color:#ffd0dd; }} .footer-link {{ text-align:center; margin-top:28px; }} .error {{ margin-top:15vh; }}
@media (max-width:700px) {{ main {{ padding-top:30px; }} .grid.two,.actions {{ grid-template-columns:1fr; }} .track {{ grid-template-columns:repeat(5,1fr); }} .topbar {{ align-items:start; }} }}
</style>
</head>
<body><main>{content}</main></body>
</html>"#,
        title = html_escape(title),
        head_extra = head_extra,
        content = content,
    )
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piet_oracle_outputs_three() {
        let source = "P3\n5 1\n255\n255 192 192 255 192 192 255 192 192 255 0 0 192 0 192\n";
        assert_eq!(run_piet_source(source).unwrap(), 3);
    }

    #[test]
    fn form_decoding_handles_spaces_and_utf8() {
        let form = parse_form("name=Luis+Benedikt&room=ABC123");
        assert_eq!(form.get("name").unwrap(), "Luis Benedikt");
        assert_eq!(form.get("room").unwrap(), "ABC123");
    }

    #[test]
    fn boost_without_energy_is_small() {
        let segment = TrackSegment {
            terrain: "jump".to_string(),
            speed: 3,
            boost: 4,
            recovery: 1,
        };
        assert_eq!(movement_for(RaceAction::Boost, 0, &segment, 3).0, 1);
        assert!(movement_for(RaceAction::Boost, 5, &segment, 3).0 > 1);
    }
}
