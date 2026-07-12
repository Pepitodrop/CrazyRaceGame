from pathlib import Path
import re


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    if text.count(old) != 1:
        raise SystemExit(f"expected exactly one match in {path}: {old[:80]!r}; got {text.count(old)}")
    file.write_text(text.replace(old, new, 1))


def replace_regex(path: str, pattern: str, replacement: str) -> None:
    file = Path(path)
    text = file.read_text()
    updated, count = re.subn(pattern, replacement, text, count=1, flags=re.S)
    if count != 1:
        raise SystemExit(f"expected one regex match in {path}: {pattern!r}; got {count}")
    file.write_text(updated)


# --- Rust routing and per-room track selection ---------------------------------
replace_once(
    "src/part1.rs",
    '''        ("GET", "/health") => Ok((
            "200 OK",
            "text/plain; charset=utf-8",
            "ok".to_string(),
            Vec::new(),
        )),''',
    '''        ("GET", "/favicon.svg") => Ok((
            "200 OK",
            "image/svg+xml; charset=utf-8",
            render_favicon_svg(),
            Vec::new(),
        )),
        ("GET", "/health") => Ok((
            "200 OK",
            "text/plain; charset=utf-8",
            "ok".to_string(),
            Vec::new(),
        )),''',
)

replace_once(
    "src/part1.rs",
    '''fn first_announcement(app: &AppState) -> String {
    app.announcements
        .first()
        .cloned()
        .unwrap_or_else(|| "Race ready.".to_string())
}
''',
    '''fn first_announcement(app: &AppState) -> String {
    app.announcements
        .first()
        .cloned()
        .unwrap_or_else(|| "Race ready.".to_string())
}

fn room_track(app: &AppState, code: &str) -> Result<Vec<TrackSegment>, RouteError> {
    let seed = app
        .rooms
        .get(code)
        .ok_or(("404 Not Found", "Room not found.".to_string()))?
        .seed;
    Ok(track_for_room(&app.track, seed))
}
''',
)

part1 = Path("src/part1.rs")
text = part1.read_text()
if text.count("    let track = app.track.clone();") != 4:
    raise SystemExit("expected four global track clones")
part1.write_text(text.replace("    let track = app.track.clone();", "    let track = room_track(&app, &code)?;"))

replace_once(
    "src/part1.rs",
    '''    if room.players[player_index].submitted.is_some() {
        return Err((
            "409 Conflict",
            "Your move is already locked in.".to_string(),
        ));
    }

    room.players[player_index].submitted = Some(action);''',
    '''    if room.players[player_index].submitted.is_some() {
        return Err((
            "409 Conflict",
            "Your move is already locked in.".to_string(),
        ));
    }
    validate_action_energy(action, room.players[player_index].energy)?;

    room.players[player_index].submitted = Some(action);''',
)

replace_once(
    "src/part1.rs",
    '''    let turn = room.local_turn.min(1);
    if room.players[turn].submitted.is_some() {
        return Err((
            "409 Conflict",
            "That move is already locked in.".to_string(),
        ));
    }
    room.players[turn].submitted = Some(action);''',
    '''    let turn = room.local_turn.min(1);
    if room.players[turn].submitted.is_some() {
        return Err((
            "409 Conflict",
            "That move is already locked in.".to_string(),
        ));
    }
    validate_action_energy(action, room.players[turn].energy)?;
    room.players[turn].submitted = Some(action);''',
)

replace_regex(
    "src/part1.rs",
    r'''fn rematch\(.*?\n\}\n\nfn ensure_local_access''',
    '''fn rematch(
    request: &HttpRequest,
    state: &Arc<Mutex<AppState>>,
) -> Result<RouteResponse, RouteError> {
    let code = clean_room_code(&required_body(request, "room")?)?;
    let token = required_body(request, "token")?;
    let mut app = state.lock().map_err(|_| internal_error())?;

    let mode = {
        let room = app
            .rooms
            .get(&code)
            .ok_or(("404 Not Found", "Room not found.".to_string()))?;
        let allowed = match room.mode {
            GameMode::Online => room.players.iter().any(|player| player.token == token),
            GameMode::Local => room.local_token.as_deref() == Some(token.as_str()),
        };
        if !allowed {
            return Err(("403 Forbidden", "Invalid racer token.".to_string()));
        }
        room.mode
    };

    let next_seed = next_random(&mut app);
    let room = app
        .rooms
        .get_mut(&code)
        .ok_or(("404 Not Found", "Room not found.".to_string()))?;
    for player in &mut room.players {
        player.distance = 0;
        player.energy = 5;
        player.submitted = None;
    }
    room.seed = next_seed;
    room.local_turn = 0;
    room.round = 1;
    room.winner = None;
    room.last_activity = now_epoch();
    room.last_summary = "Rematch started on a newly shuffled circuit. Choose your move.".to_string();

    Ok(match mode {
        GameMode::Online => redirect_to_game(&code, &token),
        GameMode::Local => redirect_to_local_game(&code, &token),
    })
}

fn ensure_local_access''',
)

# --- Core game rules ------------------------------------------------------------
replace_regex(
    "src/part2.rs",
    r'''fn resolve_round\(.*?\n\}\n\nfn generate_track''',
    '''fn resolve_round(
    room: &mut Room,
    track: &[TrackSegment],
    piet_boost: i32,
    announcements: &[String],
) {
    let mut summaries = Vec::new();
    let finish_line = track.len() as i32;
    let mut finish_scores = [0_i32; 2];

    for (index, player) in room.players.iter_mut().enumerate() {
        let action = player.submitted.take().unwrap_or(RaceAction::Drift);
        let segment_index = player.distance.clamp(0, finish_line.saturating_sub(1)) as usize;
        let segment = &track[segment_index];
        let (movement, energy_delta, note) =
            movement_for(action, player.energy, segment, piet_boost);
        let raw_distance = player.distance.saturating_add(movement);
        finish_scores[index] = raw_distance;
        player.distance = raw_distance.min(finish_line);
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

    let first_finished = finish_scores[0] >= finish_line;
    let second_finished = finish_scores[1] >= finish_line;
    room.winner = match (first_finished, second_finished) {
        (true, false) => Some(0),
        (false, true) => Some(1),
        (true, true) => match finish_scores[0].cmp(&finish_scores[1]) {
            std::cmp::Ordering::Greater => Some(0),
            std::cmp::Ordering::Less => Some(1),
            std::cmp::Ordering::Equal => {
                summaries.push("The racers reached the finish equally; the deterministic photo-finish rule decided the winner".to_string());
                Some(((room.seed ^ room.round as u64) & 1) as usize)
            }
        },
        (false, false) => None,
    };

    if let Some(index) = room.winner {
        summaries.push(format!("{} wins the race!", room.players[index].name));
    }
    room.last_summary = format!("Round {}: {}.", room.round, summaries.join(" "));
    if !announcements.is_empty() {
        let index =
            ((room.seed.wrapping_add(room.round as u64)) % announcements.len() as u64) as usize;
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
        RaceAction::Accelerate if energy >= 1 => {
            ((2 + segment.speed).clamp(1, 6), -1, "")
        }
        RaceAction::Accelerate => (1, 1, " but had no energy and coasted"),
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

fn validate_action_energy(action: RaceAction, energy: i32) -> Result<(), RouteError> {
    match action {
        RaceAction::Accelerate if energy < 1 => Err((
            "409 Conflict",
            "Accelerate requires 1 energy. Choose Drift to recover energy.".to_string(),
        )),
        RaceAction::Boost if energy < 3 => Err((
            "409 Conflict",
            "Piet Boost requires 3 energy. Choose Drift to recover energy.".to_string(),
        )),
        _ => Ok(()),
    }
}

fn track_for_room(base: &[TrackSegment], seed: u64) -> Vec<TrackSegment> {
    let mut track = base.to_vec();
    if track.len() <= 3 {
        return track;
    }

    let mut state = seed ^ 0xA076_1D64_78BD_642F;
    for index in (2..track.len() - 1).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let target = 1 + (state as usize % index);
        track.swap(index, target);
    }
    track
}

fn generate_track''',
)

# --- Security policy for favicon/data imagery ----------------------------------
replace_once(
    "src/part3.rs",
    '''    "Content-Security-Policy: default-src 'none'; style-src 'unsafe-inline'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'\\r\\n"''',
    '''    "Content-Security-Policy: default-src 'none'; style-src 'unsafe-inline'; img-src 'self' data:; form-action 'self'; base-uri 'none'; frame-ancestors 'none'\\r\\n"''',
)
