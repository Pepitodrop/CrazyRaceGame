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


PART4 = r'''fn render_landing_page() -> String {
    page_shell(
        "Crazy Race",
        "",
        r#"
<section class="hero">
  <p class="eyebrow">R × Rust × TrumpScript × Piet</p>
  <h1>Crazy Race</h1>
  <p>A server-rendered, turn-based 1v1 race for desktop and mobile. Every new room receives a newly shuffled R-generated circuit.</p>
</section>
<section class="grid three">
  <article class="card">
    <p class="mode-tag">Online</p>
    <h2>Create a race</h2>
    <form method="post" action="/create">
      <label>Racer name<input name="name" maxlength="24" required autocomplete="nickname"></label>
      <button type="submit">Create online room</button>
    </form>
  </article>
  <article class="card">
    <p class="mode-tag">Online</p>
    <h2>Join a rival</h2>
    <form method="post" action="/join">
      <label>Room code<input name="room" maxlength="6" required autocapitalize="characters" inputmode="text"></label>
      <label>Racer name<input name="name" maxlength="24" required autocomplete="nickname"></label>
      <button type="submit">Join room</button>
    </form>
  </article>
  <article class="card local-card">
    <p class="mode-tag">Same device</p>
    <h2>Local 1v1</h2>
    <p>Take turns on one phone, tablet, or computer. Player 1’s choice stays hidden while the device is passed.</p>
    <form method="post" action="/local">
      <label>Player 1<input name="player_one" maxlength="24" required autocomplete="off"></label>
      <label>Player 2<input name="player_two" maxlength="24" required autocomplete="off"></label>
      <button type="submit">Start local race</button>
    </form>
  </article>
</section>
<section class="card stack-note">
  <h2>Four-language engine</h2>
  <p><strong>R</strong> generates the circuit data. <strong>Rust</strong> shuffles a separate circuit per room and owns networking and game state. <strong>TrumpScript</strong> runs the announcer through its original Python interpreter. <strong>Piet</strong> paints the boost value.</p>
</section>
"#,
    )
}

fn render_online_game_page(
    room: &Room,
    player_index: usize,
    track: &[TrackSegment],
    piet_boost: i32,
) -> String {
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
    let player_card = render_racer(player, track, true);
    let opponent_card = opponent
        .map(|rival| render_racer(rival, track, false))
        .unwrap_or_else(|| {
            "<article class=\"racer card muted\"><h3>Waiting for rival…</h3><p>Share the room code.</p></article>".to_string()
        });

    let actions = if waiting_for_player {
        "<p class=\"notice\">Waiting for the second racer. This page refreshes automatically.</p>"
            .to_string()
    } else if room.winner.is_some() {
        render_rematch_form(room, &player.token)
    } else if player.submitted.is_some() {
        "<p class=\"notice\">Move locked. Waiting for your rival; this page refreshes automatically.</p>"
            .to_string()
    } else {
        render_action_form(
            "/action",
            room,
            &player.token,
            piet_boost,
            player.energy,
            None,
        )
    };

    let winner_banner = winner_banner(room, Some(player_index));
    let content = render_game_layout(
        room,
        track,
        GameLayout {
            winner_banner: &winner_banner,
            first_card: &player_card,
            second_card: &opponent_card,
            actions: &actions,
            action_title: "Your move",
            mode_label: "Online room",
        },
    );
    page_shell("Crazy Race", refresh, &content)
}

fn render_local_game_page(
    room: &Room,
    token: &str,
    track: &[TrackSegment],
    piet_boost: i32,
) -> String {
    let current = room.local_turn.min(1);
    let first_card = render_racer(&room.players[0], track, current == 0);
    let second_card = render_racer(&room.players[1], track, current == 1);
    let actions = if room.winner.is_some() {
        render_rematch_form(room, token)
    } else {
        let prompt = if current == 0 {
            format!("{} chooses first", html_escape(&room.players[0].name))
        } else {
            format!(
                "Pass the device to {}. {}’s move is hidden.",
                html_escape(&room.players[1].name),
                html_escape(&room.players[0].name)
            )
        };
        format!(
            "<div class=\"handover\"><p>{prompt}</p></div>{}",
            render_action_form(
                "/local-action",
                room,
                token,
                piet_boost,
                room.players[current].energy,
                Some(current),
            )
        )
    };

    let winner_banner = winner_banner(room, None);
    let action_title = if room.winner.is_some() {
        "Local race finished"
    } else if current == 0 {
        "Player 1 move"
    } else {
        "Player 2 move"
    };
    let content = render_game_layout(
        room,
        track,
        GameLayout {
            winner_banner: &winner_banner,
            first_card: &first_card,
            second_card: &second_card,
            actions: &actions,
            action_title,
            mode_label: "Local pass-and-play",
        },
    );
    page_shell("Crazy Race · Local", "", &content)
}

struct GameLayout<'a> {
    winner_banner: &'a str,
    first_card: &'a str,
    second_card: &'a str,
    actions: &'a str,
    action_title: &'a str,
    mode_label: &'a str,
}

fn render_game_layout(room: &Room, track: &[TrackSegment], layout: GameLayout<'_>) -> String {
    let GameLayout {
        winner_banner,
        first_card,
        second_card,
        actions,
        action_title,
        mode_label,
    } = layout;
    format!(
        r#"
<section class="topbar">
  <div><p class="eyebrow">{mode_label}</p><h1>{code}</h1></div>
  <div class="round">Round {round}</div>
</section>
{winner_banner}
<section class="grid two racers">
  {first_card}
  {second_card}
</section>
<section class="card announcer">
  <p class="eyebrow">TrumpScript announcer</p>
  <blockquote>“{announcement}”</blockquote>
  <p>{summary}</p>
</section>
<section class="card track-card">
  <h2>R-generated circuit</h2>
  <p class="section-help">Each tile is one segment. The symbol and color show its terrain; P1 and P2 mark the racers’ current segments.</p>
  <div class="track-scroll"><div class="track" data-track-signature="{track_signature}">{track_html}</div></div>
  {legend}
</section>
<section class="card action-card">
  <h2>{action_title}</h2>
  {actions}
</section>
{piet_easter_egg}
<p class="footer-link"><a href="/">Leave race</a></p>
"#,
        mode_label = html_escape(mode_label),
        code = html_escape(&room.code),
        round = room.round,
        winner_banner = winner_banner,
        first_card = first_card,
        second_card = second_card,
        announcement = html_escape(&room.announcement),
        summary = html_escape(&room.last_summary),
        track_signature = html_escape(&track_signature(track)),
        track_html = render_track(track, room),
        legend = render_track_legend(),
        action_title = html_escape(action_title),
        actions = actions,
        piet_easter_egg = render_piet_easter_egg(),
    )
}

fn winner_banner(room: &Room, perspective: Option<usize>) -> String {
    room.winner
        .map(|winner| {
            let message = match perspective {
                Some(index) if index == winner => "You won the race.".to_string(),
                Some(_) => format!("{} won the race.", html_escape(&room.players[winner].name)),
                None => format!(
                    "{} wins the local race!",
                    html_escape(&room.players[winner].name)
                ),
            };
            format!("<div class=\"winner\">{message}</div>")
        })
        .unwrap_or_default()
}

fn render_racer(player: &Player, track: &[TrackSegment], highlighted: bool) -> String {
    let finish_line = track.len() as i32;
    let displayed_distance = player.distance.clamp(0, finish_line);
    let progress = if finish_line <= 0 {
        0
    } else {
        ((displayed_distance * 100) / finish_line).clamp(0, 100)
    };
    let terrain = if displayed_distance >= finish_line {
        "Finished"
    } else {
        track
            .get(displayed_distance as usize)
            .map(|segment| terrain_label(&segment.terrain))
            .unwrap_or("Unknown")
    };
    let energy = player.energy.clamp(0, 10);
    let energy_progress = energy * 10;
    format!(
        r#"<article class="racer card {highlight}">
<h3>{name}</h3>
<div class="racer-metrics">
  <div class="metric"><span>Progress</span><strong>Segment {distance} of {finish}</strong></div>
  <div class="metric"><span>Current terrain</span><strong>{terrain}</strong></div>
</div>
<div class="progress" role="progressbar" aria-label="Race progress" aria-valuemin="0" aria-valuemax="100" aria-valuenow="{progress}"><span style="width:{progress}%"></span></div>
<div class="energy-row"><span>Energy</span><strong>{energy} / 10</strong></div>
<div class="energy-meter" role="progressbar" aria-label="Energy" aria-valuemin="0" aria-valuemax="10" aria-valuenow="{energy}"><span style="width:{energy_progress}%"></span></div>
</article>"#,
        highlight = if highlighted { "active-racer" } else { "" },
        name = html_escape(&player.name),
        distance = displayed_distance,
        finish = finish_line,
        terrain = terrain,
        progress = progress,
        energy = energy,
        energy_progress = energy_progress,
    )
}

fn render_action_form(
    endpoint: &str,
    room: &Room,
    token: &str,
    piet_boost: i32,
    energy: i32,
    local_player: Option<usize>,
) -> String {
    let player_hint = local_player
        .map(|index| {
            format!(
                "<p class=\"current-player\">Choosing for <strong>{}</strong></p>",
                html_escape(&room.players[index].name)
            )
        })
        .unwrap_or_default();
    let accelerate_disabled = if energy < 1 {
        "disabled aria-disabled=\"true\""
    } else {
        ""
    };
    let accelerate_detail = if energy < 1 {
        "Needs 1 energy · Drift to recharge"
    } else {
        "Reliable speed · costs 1 energy"
    };
    let boost_disabled = if energy < 3 {
        "disabled aria-disabled=\"true\""
    } else {
        ""
    };
    let boost_detail = if energy < 3 {
        "Needs 3 energy · Drift to recharge"
    } else {
        "Up to +oracle power · costs 3 energy"
    };
    format!(
        r#"{player_hint}
<p class="energy-help">Available energy: <strong>{energy} / 10</strong>. Drift is always available and recharges energy.</p>
<form method="post" action="{endpoint}" class="actions">
<input type="hidden" name="room" value="{room}">
<input type="hidden" name="token" value="{token}">
<button type="submit" name="action" value="accelerate" {accelerate_disabled}><strong>Accelerate</strong><span>{accelerate_detail}</span></button>
<button type="submit" name="action" value="drift"><strong>Drift</strong><span>Best on curves · recovers energy</span></button>
<button type="submit" name="action" value="boost" {boost_disabled}><strong>Piet Boost</strong><span>{boost_detail} ({boost})</span></button>
</form>"#,
        player_hint = player_hint,
        energy = energy.clamp(0, 10),
        endpoint = html_escape(endpoint),
        room = html_escape(&room.code),
        token = html_escape(token),
        accelerate_disabled = accelerate_disabled,
        accelerate_detail = accelerate_detail,
        boost_disabled = boost_disabled,
        boost_detail = boost_detail,
        boost = piet_boost,
    )
}

fn render_rematch_form(room: &Room, token: &str) -> String {
    format!(
        r#"<form method="post" action="/rematch" class="actions single-action">
<input type="hidden" name="room" value="{room}">
<input type="hidden" name="token" value="{token}">
<button type="submit"><strong>Start rematch</strong><span>Reset both racers and shuffle a new circuit</span></button>
</form>"#,
        room = html_escape(&room.code),
        token = html_escape(token)
    )
}

fn terrain_symbol(terrain: &str) -> &'static str {
    match terrain {
        "straight" => "→",
        "curve" => "↪",
        "mud" => "≈",
        "jump" => "▲",
        _ => "?",
    }
}

fn terrain_label(terrain: &str) -> &'static str {
    match terrain {
        "straight" => "Straight",
        "curve" => "Curve",
        "mud" => "Mud",
        "jump" => "Jump",
        _ => "Unknown",
    }
}

fn track_signature(track: &[TrackSegment]) -> String {
    track
        .iter()
        .map(|segment| match segment.terrain.as_str() {
            "straight" => 'S',
            "curve" => 'C',
            "mud" => 'M',
            "jump" => 'J',
            _ => '?',
        })
        .collect()
}

fn render_track(track: &[TrackSegment], room: &Room) -> String {
    let mut output = String::new();
    let finish_line = track.len() as i32;
    for (index, segment) in track.iter().enumerate() {
        let terrain = html_escape(&segment.terrain);
        let mut markers = String::new();
        for (player_index, player) in room.players.iter().enumerate() {
            let marker_index = if player.distance >= finish_line {
                track.len().saturating_sub(1)
            } else {
                player.distance.max(0) as usize
            };
            if marker_index == index {
                std::fmt::Write::write_fmt(
                    &mut markers,
                    format_args!(
                        "<b class=\"player-marker marker-{}\">P{}</b>",
                        player_index + 1,
                        player_index + 1
                    ),
                )
                .expect("writing to a String cannot fail");
            }
        }
        std::fmt::Write::write_fmt(
            &mut output,
            format_args!(
                "<span class=\"segment {terrain}\" title=\"{}: speed {:+}, boost +{}, recovery +{}\"><span class=\"player-markers\">{markers}</span><span class=\"terrain-symbol\">{}</span><span class=\"segment-number\">{}</span></span>",
                terrain_label(&segment.terrain),
                segment.speed,
                segment.boost,
                segment.recovery,
                terrain_symbol(&segment.terrain),
                index + 1
            ),
        )
        .expect("writing to a String cannot fail");
    }
    output
}

fn render_track_legend() -> String {
    r#"<div class="legend" aria-label="Terrain legend">
<span><i class="legend-swatch straight">→</i><span><strong>Straight</strong><small>Usually fastest</small></span></span>
<span><i class="legend-swatch curve">↪</i><span><strong>Curve</strong><small>Drift bonus</small></span></span>
<span><i class="legend-swatch mud">≈</i><span><strong>Mud</strong><small>Usually slower</small></span></span>
<span><i class="legend-swatch jump">▲</i><span><strong>Jump</strong><small>Boost bonus</small></span></span>
</div>"#.to_string()
}

fn render_piet_easter_egg() -> String {
    r#"<details class="card piet-easter-egg">
<summary>🥚 Reveal the Piet boost program</summary>
<p>This magnified five-codel painting is the executable Piet source used by the game. It pushes <strong>3</strong> and outputs that number as the boost oracle.</p>
<svg class="piet-painting" viewBox="0 0 500 100" role="img" aria-label="Piet program with three light-red codels, one red codel, and one dark-magenta codel" shape-rendering="crispEdges">
  <rect x="0" y="0" width="100" height="100" fill="#ffc0c0"/>
  <rect x="100" y="0" width="100" height="100" fill="#ffc0c0"/>
  <rect x="200" y="0" width="100" height="100" fill="#ffc0c0"/>
  <rect x="300" y="0" width="100" height="100" fill="#ff0000"/>
  <rect x="400" y="0" width="100" height="100" fill="#c000c0"/>
</svg>
<code>push 3 → out(number)</code>
</details>"#.to_string()
}

fn render_favicon_svg() -> String {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">
<defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1"><stop stop-color="#6fe7ff"/><stop offset="1" stop-color="#b38cff"/></linearGradient></defs>
<rect width="64" height="64" rx="14" fill="#0b1020"/>
<path d="M13 39h7l5-12h22l5 12h3v9h-5a7 7 0 0 1-14 0H28a7 7 0 0 1-14 0h-1z" fill="url(#g)"/>
<circle cx="21" cy="48" r="4" fill="#0b1020"/><circle cx="43" cy="48" r="4" fill="#0b1020"/>
<path d="M29 30h14l3 7H26z" fill="#0b1020" opacity=".8"/>
</svg>"##.to_string()
}

fn render_error_page(message: &str) -> String {
    page_shell(
        "Crazy Race · Error",
        "",
        &format!(
            "<section class=\"card error\"><p class=\"eyebrow\">Something went wrong</p><h1>Race interrupted</h1><p>{}</p><a class=\"button-link\" href=\"/\">Back to start</a></section>",
            html_escape(message)
        ),
    )
}
'''
Path("src/part4.rs").write_text(PART4)
