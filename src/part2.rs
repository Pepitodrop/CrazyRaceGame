fn resolve_round(
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
        RaceAction::Accelerate if energy >= 1 => ((2 + segment.speed).clamp(1, 6), -1, ""),
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
    let content =
        fs::read_to_string(path).map_err(|error| format!("cannot read {path}: {error}"))?;
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
            speed: fields[2]
                .parse()
                .map_err(|_| format!("invalid speed at line {}", line_number + 1))?,
            boost: fields[3]
                .parse()
                .map_err(|_| format!("invalid boost at line {}", line_number + 1))?,
            recovery: fields[4]
                .parse()
                .map_err(|_| format!("invalid recovery at line {}", line_number + 1))?,
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
    let source =
        fs::read_to_string(path).map_err(|error| format!("cannot read {path}: {error}"))?;
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
        (255, 192, 192) => PietColor::Chromatic {
            hue: 0,
            lightness: 0,
        },
        (255, 0, 0) => PietColor::Chromatic {
            hue: 0,
            lightness: 1,
        },
        (192, 0, 0) => PietColor::Chromatic {
            hue: 0,
            lightness: 2,
        },
        (255, 255, 192) => PietColor::Chromatic {
            hue: 1,
            lightness: 0,
        },
        (255, 255, 0) => PietColor::Chromatic {
            hue: 1,
            lightness: 1,
        },
        (192, 192, 0) => PietColor::Chromatic {
            hue: 1,
            lightness: 2,
        },
        (192, 255, 192) => PietColor::Chromatic {
            hue: 2,
            lightness: 0,
        },
        (0, 255, 0) => PietColor::Chromatic {
            hue: 2,
            lightness: 1,
        },
        (0, 192, 0) => PietColor::Chromatic {
            hue: 2,
            lightness: 2,
        },
        (192, 255, 255) => PietColor::Chromatic {
            hue: 3,
            lightness: 0,
        },
        (0, 255, 255) => PietColor::Chromatic {
            hue: 3,
            lightness: 1,
        },
        (0, 192, 192) => PietColor::Chromatic {
            hue: 3,
            lightness: 2,
        },
        (192, 192, 255) => PietColor::Chromatic {
            hue: 4,
            lightness: 0,
        },
        (0, 0, 255) => PietColor::Chromatic {
            hue: 4,
            lightness: 1,
        },
        (0, 0, 192) => PietColor::Chromatic {
            hue: 4,
            lightness: 2,
        },
        (255, 192, 255) => PietColor::Chromatic {
            hue: 5,
            lightness: 0,
        },
        (255, 0, 255) => PietColor::Chromatic {
            hue: 5,
            lightness: 1,
        },
        (192, 0, 192) => PietColor::Chromatic {
            hue: 5,
            lightness: 2,
        },
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
