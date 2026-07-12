#!/usr/bin/env python3
from pathlib import Path


def replace_between(path: str, start: str, end: str, replacement: str) -> None:
    file = Path(path)
    text = file.read_text()
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    file.write_text(text[:start_index] + replacement + text[end_index:])


part2_replacement = r'''#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PietOracleResult {
    value: i32,
    round_input: i32,
    terrain_input: i32,
    comeback_input: i32,
    energy_input: i32,
}

fn resolve_round(
    room: &mut Room,
    track: &[TrackSegment],
    piet_fallback: i32,
    announcements: &[String],
) {
    let mut summaries = Vec::new();
    let finish_line = track.len() as i32;
    let round = room.round;
    let snapshot: Vec<(i32, i32)> = room
        .players
        .iter()
        .map(|player| (player.distance, player.energy))
        .collect();

    for (player_index, player) in room.players.iter_mut().enumerate() {
        let action = player.submitted.take().unwrap_or(RaceAction::Accelerate);
        let segment_index = player.distance.clamp(0, finish_line.saturating_sub(1)) as usize;
        let segment = &track[segment_index];
        let oracle = adaptive_piet_oracle(
            round,
            player_index,
            &snapshot,
            segment,
            player.energy,
            piet_fallback,
        );
        let boost_was_available = action == RaceAction::Boost && player.energy >= 3;
        let (movement, energy_delta, note) =
            movement_for(action, player.energy, segment, oracle.value);
        player.distance += movement;
        player.energy = (player.energy + energy_delta).clamp(0, 10);
        let oracle_note = if boost_was_available {
            format!(" (adaptive Piet +{})", oracle.value)
        } else {
            String::new()
        };
        summaries.push(format!(
            "{} used {} on {} and moved {} segment{}{}{}",
            player.name,
            action.label(),
            segment.terrain,
            movement,
            if movement == 1 { "" } else { "s" },
            note,
            oracle_note
        ));
    }

    let first_finished = room.players[0].distance >= finish_line;
    let second_finished = room.players[1].distance >= finish_line;
    room.winner = match (first_finished, second_finished) {
        (true, false) => Some(0),
        (false, true) => Some(1),
        (true, true) => match room.players[0].distance.cmp(&room.players[1].distance) {
            std::cmp::Ordering::Greater => Some(0),
            std::cmp::Ordering::Less => Some(1),
            std::cmp::Ordering::Equal => Some(((room.seed ^ room.round as u64) & 1) as usize),
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

fn adaptive_piet_oracle(
    round: u32,
    player_index: usize,
    snapshot: &[(i32, i32)],
    segment: &TrackSegment,
    energy: i32,
    fallback: i32,
) -> PietOracleResult {
    let own_distance = snapshot
        .get(player_index)
        .map(|state| state.0)
        .unwrap_or_default();
    let opponent_distance = snapshot
        .get(1usize.saturating_sub(player_index))
        .map(|state| state.0)
        .unwrap_or(own_distance);
    let round_input = round.min(i32::MAX as u32) as i32;
    let terrain_input = terrain_code(&segment.terrain);
    let comeback_input = ((opponent_distance - own_distance).max(0) / 4).clamp(0, 3);
    let energy_input = (energy.clamp(0, 10) / 3).clamp(0, 3);
    let inputs = [round_input, terrain_input, comeback_input, energy_input];
    let value = cached_piet_program()
        .and_then(|source| run_piet_source_with_inputs(source, &inputs))
        .unwrap_or(fallback)
        .clamp(1, 4);

    PietOracleResult {
        value,
        round_input,
        terrain_input,
        comeback_input,
        energy_input,
    }
}

fn terrain_code(terrain: &str) -> i32 {
    match terrain {
        "curve" => 1,
        "mud" => 2,
        "jump" => 3,
        _ => 0,
    }
}

'''
replace_between("src/part2.rs", "fn resolve_round(", "fn generate_track(", part2_replacement)

piet_replacement = r'''fn load_piet_program(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("cannot read {path}: {error}"))
}

fn cached_piet_program() -> Result<&'static str, String> {
    static SOURCE: std::sync::OnceLock<Result<String, String>> = std::sync::OnceLock::new();
    SOURCE
        .get_or_init(|| load_piet_program(PIET_PROGRAM))
        .as_deref()
        .map_err(str::to_string)
}

fn run_piet_oracle(path: &str) -> Result<i32, String> {
    let source = load_piet_program(path)?;
    run_piet_source_with_inputs(&source, &[1, 0, 0, 1])
}

fn run_piet_source(source: &str) -> Result<i32, String> {
    run_piet_source_with_inputs(source, &[])
}

fn run_piet_source_with_inputs(source: &str, inputs: &[i32]) -> Result<i32, String> {
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
    let mut input_index = 0usize;
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
                (0, 0) => {}
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
                (3, 0) => binary_stack_op(&mut stack, |a, b| if a > b { 1 } else { 0 }),
                (4, 0) => {
                    if let Some(value) = stack.last().copied() {
                        stack.push(value);
                    }
                }
                (4, 2) => {
                    let value = inputs
                        .get(input_index)
                        .copied()
                        .ok_or_else(|| format!("Piet requested missing numeric input {input_index}"))?;
                    input_index += 1;
                    stack.push(value);
                }
                (5, 1) => {
                    if let Some(value) = stack.pop() {
                        output.push(value);
                    }
                }
                command => {
                    return Err(format!(
                        "unsupported linear Piet command ({}, {})",
                        command.0, command.1
                    ));
                }
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

'''
replace_between("src/part2.rs", "fn run_piet_oracle(", "fn binary_stack_op", piet_replacement)

part4 = Path("src/part4.rs")
text = part4.read_text()
text = text.replace(
    "<p><strong>R</strong> generates the circuit. <strong>Rust</strong> owns networking and game state. <strong>TrumpScript</strong> runs the announcer through its original Python interpreter. <strong>Piet</strong> paints the boost value.</p>",
    "<p><strong>R</strong> generates the circuit. <strong>Rust</strong> owns networking and game state. <strong>TrumpScript</strong> runs the announcer through its original Python interpreter. <strong>Piet</strong> reads the live round, terrain, comeback gap, and energy state to calculate a changing boost.</p>",
)
old = '''    let boost_disabled = if player.energy < 3 {
        "disabled aria-disabled=\\"true\\" title=\\"Piet Boost needs 3 energy\\""
    } else {
        ""
    };
    format!(
        r#"{player_hint}
<div class="turn-status"><span><strong>Current:</strong> segment {segment_number}/{finish}</span><span class="terrain-pill {terrain}">{symbol} {terrain_name}</span><span><strong>Energy:</strong> {energy}/10</span></div>
<form method="post" action="{endpoint}" class="actions">
<input type="hidden" name="room" value="{room}">
<input type="hidden" name="token" value="{token}">
<button type="submit" name="action" value="accelerate"><strong>Accelerate</strong><span>Base 2 + terrain speed · costs 1 energy</span></button>
<button type="submit" name="action" value="drift"><strong>Drift</strong><span>Curve bonus · restores this segment’s recovery energy</span></button>
<button type="submit" name="action" value="boost" {boost_disabled}><strong>Piet Boost</strong><span>Oracle +{piet_boost} and terrain bonus · costs 3 energy{boost_note}</span></button>
</form>"#,
'''
new = '''    let snapshot: Vec<(i32, i32)> = room
        .players
        .iter()
        .map(|racer| (racer.distance, racer.energy))
        .collect();
    let oracle = adaptive_piet_oracle(
        room.round,
        index,
        &snapshot,
        segment,
        player.energy,
        piet_boost,
    );
    let boost_disabled = if player.energy < 3 {
        "disabled aria-disabled=\\"true\\" title=\\"Piet Boost needs 3 energy\\""
    } else {
        ""
    };
    format!(
        r#"{player_hint}
<div class="turn-status"><span><strong>Current:</strong> segment {segment_number}/{finish}</span><span class="terrain-pill {terrain}">{symbol} {terrain_name}</span><span><strong>Energy:</strong> {energy}/10</span></div>
<form method="post" action="{endpoint}" class="actions">
<input type="hidden" name="room" value="{room}">
<input type="hidden" name="token" value="{token}">
<button type="submit" name="action" value="accelerate"><strong>Accelerate</strong><span>Base 2 + terrain speed · costs 1 energy</span></button>
<button type="submit" name="action" value="drift"><strong>Drift</strong><span>Curve bonus · restores this segment’s recovery energy</span></button>
<button type="submit" name="action" value="boost" {boost_disabled}><strong>Piet Boost · +{piet_value}</strong><span>Adaptive inputs: round {round_input}, terrain {terrain_input}, comeback {comeback_input}, energy {energy_input} · costs 3 energy{boost_note}</span></button>
</form>"#,
'''
if old not in text:
    raise SystemExit("render_action_form anchor not found")
text = text.replace(old, new)
text = text.replace("        piet_boost = piet_boost,\n", "        piet_value = oracle.value,\n        round_input = oracle.round_input,\n        terrain_input = oracle.terrain_input,\n        comeback_input = oracle.comeback_input,\n        energy_input = oracle.energy_input,\n")
part4.write_text(text)

part5 = Path("src/part5.rs")
text = part5.read_text()
text = text.replace(
    "This is the actual image program from <code>piet/boost_oracle.ppm</code>. Read left to right, it pushes and outputs the boost value used by the game.",
    "This is the actual adaptive image program from <code>piet/boost_oracle.ppm</code>. It reads round, terrain, comeback gap, and energy, then computes <code>1 + (sum mod 4)</code> for a changing +1 to +4 boost.",
)
anchor = '''    #[test]
    fn piet_oracle_outputs_three() {
        let source = "P3\\n5 1\\n255\\n255 192 192 255 192 192 255 192 192 255 0 0 192 0 192\\n";
        assert_eq!(run_piet_source(source).unwrap(), 3);
    }
'''
replacement = anchor + '''
    #[test]
    fn adaptive_piet_oracle_changes_with_inputs_and_stays_bounded() {
        let source = include_str!("../piet/boost_oracle.ppm");
        assert_eq!(run_piet_source_with_inputs(source, &[1, 0, 0, 1]).unwrap(), 3);
        assert_eq!(run_piet_source_with_inputs(source, &[2, 3, 1, 2]).unwrap(), 1);
        for round in 1..12 {
            let value = run_piet_source_with_inputs(source, &[round, 2, 3, 1]).unwrap();
            assert!((1..=4).contains(&value));
        }
    }
'''
if anchor not in text:
    raise SystemExit("part5 Piet test anchor not found")
text = text.replace(anchor, replacement)
part5.write_text(text)

Path("piet/boost_oracle.ppm").write_text("""P3
# Adaptive linear Piet oracle.
# Inputs, in order: round, terrain code, comeback bucket, energy bucket.
# It adds all four inputs, computes modulo 4, adds 1, and outputs 1..4.
16 1
255
255 192 192   0 0 192   0 255 0   0 255 255   255 255 192   192 255 192   192 0 0   192 192 0   192 192 0   192 192 0   192 192 0   255 255 192   0 255 255   0 192 192   0 0 192   192 255 255
""")

for path in ["Cargo.toml", "Cargo.lock", "Dockerfile", "README.md"]:
    file = Path(path)
    text = file.read_text().replace("1.0.3", "1.1.0")
    file.write_text(text)

readme = Path("README.md")
text = readme.read_text()
text = text.replace(
    "- Adds the Piet oracle output, currently `3`, plus the terrain speed and boost values.\n",
    "- Runs the Piet image program with four live inputs: round, terrain code, comeback gap, and energy bucket.\n- The image calculates `1 + ((round + terrain + comeback + energy) mod 4)`, producing a deterministic `+1` to `+4`.\n- The button previews the exact Piet output before submission, and Rust clamps the result to the safe range `1..=4`.\n",
)
text = text.replace(
    "`piet/boost_oracle.ppm` is executable image source. Its codel transitions push and output `3`, which Rust applies as the oracle boost modifier.",
    "`piet/boost_oracle.ppm` is executable image source. It uses Piet numeric input, stack arithmetic, modulo, push, add, and numeric output to turn live race state into a deterministic adaptive boost between `+1` and `+4`.",
)
readme.write_text(text)

changelog = Path("CHANGELOG.md")
text = changelog.read_text()
entry = """## [1.1.0] - 2026-07-13

### Added

- Adaptive Piet oracle inputs for round, terrain, comeback gap, and energy.
- Live Piet boost preview in both online and local controls.
- Piet numeric-input and comparison support in the bounded linear interpreter.

### Changed

- Piet Boost now varies deterministically from `+1` to `+4` instead of always returning a fixed `+3`.
- Round summaries report the adaptive Piet result used for successful boosts.
- The Piet easter egg and README explain the real image algorithm.

"""
text = text.replace("## [Unreleased]\n\nNo unreleased changes.\n\n", "## [Unreleased]\n\nNo unreleased changes.\n\n" + entry)
text = text.replace("[Unreleased]: https://github.com/Pepitodrop/CrazyRaceGame/compare/v1.0.2...HEAD", "[Unreleased]: https://github.com/Pepitodrop/CrazyRaceGame/compare/v1.1.0...HEAD\n[1.1.0]: https://github.com/Pepitodrop/CrazyRaceGame/compare/v1.0.3...v1.1.0")
changelog.write_text(text)

Path("docs/RELEASE_NOTES_v1.1.0.md").write_text("""# Crazy Race 1.1.0

Crazy Race 1.1.0 makes Piet an active, changing part of every race.

## Adaptive Piet oracle

The image program now reads four deterministic numeric inputs:

- current round;
- terrain code;
- comeback gap bucket;
- energy bucket.

It calculates `1 + ((round + terrain + comeback + energy) mod 4)` and outputs a boost between `+1` and `+4`. The UI previews the value before a player commits the move, while Rust validates and clamps every result.

## Interpreter improvements

The bounded linear Piet runtime now supports numeric input and comparison in addition to the existing stack arithmetic, modulo, duplicate, push, and output operations.

## Compatibility and deployment

The Docker, standard Compose, Snap-compatible local Compose, and hardened production Compose behavior are unchanged from v1.0.3. Rooms remain in memory and public deployments still require HTTPS.
""")

smoke = Path("scripts/smoke-test.sh")
text = smoke.read_text()
text = text.replace(
    "grep -q 'Local 1v1' \"$temp_dir/landing.html\"\n",
    "grep -q 'Local 1v1' \"$temp_dir/landing.html\"\ngrep -q 'Piet</strong> reads the live round' \"$temp_dir/landing.html\"\n",
)
text = text.replace(
    "grep -q 'Bob used Drift' \"$temp_dir/resolved.html\"\n",
    "grep -q 'Bob used Drift' \"$temp_dir/resolved.html\"\ngrep -q 'Piet Boost · +' \"$temp_dir/resolved.html\"\n",
)
smoke.write_text(text)
