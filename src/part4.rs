fn render_landing_page() -> String {
    page_shell(
        "Crazy Race",
        "",
        r#"
<section class="hero">
  <p class="eyebrow">R × Rust × TrumpScript × Piet</p>
  <h1>Crazy Race</h1>
  <p>A server-rendered, turn-based 1v1 race for desktop and mobile. Play online or share one device.</p>
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
  <h2>How the race works</h2>
  <p>Reach segment 20 first. Accelerate spends 1 energy, Drift restores energy and is strongest on curves, and Piet Boost spends 3 energy for the largest move. The track stays fixed during one race but a fresh default circuit is generated whenever the container starts.</p>
</section>
<section class="card stack-note">
  <h2>Four-language engine</h2>
  <p><strong>R</strong> generates the circuit. <strong>Rust</strong> owns networking and game state. <strong>TrumpScript</strong> runs the announcer through its original Python interpreter. <strong>Piet</strong> paints the boost value.</p>
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
    let finish_line = track.len() as i32;
    let player_card = render_racer(player, finish_line, true);
    let opponent_card = opponent
        .map(|rival| render_racer(rival, finish_line, false))
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
            Some(player_index),
            track,
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
    let finish_line = track.len() as i32;
    let current = room.local_turn.min(1);
    let first_card = render_racer(&room.players[0], finish_line, current == 0);
    let second_card = render_racer(&room.players[1], finish_line, current == 1);
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
                Some(current),
                track,
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
  <div class="track-heading"><div><h2>R-generated circuit</h2><p>Track ID <code>{track_id}</code> · reach segment {finish}</p></div><span class="track-help">Symbol + number = terrain + segment</span></div>
  <div class="track-scroll"><div class="track">{track_html}</div></div>
  <div class="legend" aria-label="Track terrain legend">
    <span class="legend-item straight"><b>▰</b><span><strong>Straight</strong><small>fast acceleration</small></span></span>
    <span class="legend-item curve"><b>◒</b><span><strong>Curve</strong><small>best for drift</small></span></span>
    <span class="legend-item mud"><b>≈</b><span><strong>Mud</strong><small>slows movement</small></span></span>
    <span class="legend-item jump"><b>▲</b><span><strong>Jump</strong><small>boost bonus</small></span></span>
  </div>
</section>
<section class="card action-card">
  <h2>{action_title}</h2>
  {actions}
</section>
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
        track_id = track_fingerprint(track),
        finish = track.len(),
        track_html = render_track(track),
        action_title = html_escape(action_title),
        actions = actions,
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

fn render_racer(player: &Player, finish_line: i32, highlighted: bool) -> String {
    let shown_distance = player.distance.clamp(0, finish_line.max(0));
    let progress = if finish_line <= 0 {
        0
    } else {
        ((shown_distance * 100) / finish_line).clamp(0, 100)
    };
    let segment_label = if shown_distance >= finish_line {
        format!("Finished · score {}", player.distance.max(finish_line))
    } else {
        format!("Segment {} of {}", shown_distance + 1, finish_line)
    };
    format!(
        r#"<article class="racer card {highlight}">
<h3>{name}</h3>
<div class="stats"><span><strong>{shown_distance}</strong> / {finish}</span><span><strong>{energy}</strong> / 10 energy</span></div>
<div class="status-labels"><span>{segment_label}</span><span>{energy_hint}</span></div>
<div class="progress" role="progressbar" aria-label="Race progress" aria-valuemin="0" aria-valuemax="100" aria-valuenow="{progress}"><span style="width:{progress}%"></span></div>
<div class="energy-bar" role="progressbar" aria-label="Energy" aria-valuemin="0" aria-valuemax="10" aria-valuenow="{energy}"><span style="width:{energy_percent}%"></span></div>
</article>"#,
        highlight = if highlighted { "active-racer" } else { "" },
        name = html_escape(&player.name),
        shown_distance = shown_distance,
        finish = finish_line,
        energy = player.energy,
        energy_percent = player.energy.clamp(0, 10) * 10,
        segment_label = html_escape(&segment_label),
        energy_hint = if player.energy >= 3 {
            "Piet Boost ready"
        } else {
            "Need 3 energy for Piet Boost"
        },
        progress = progress,
    )
}

fn render_action_form(
    endpoint: &str,
    room: &Room,
    token: &str,
    piet_boost: i32,
    player_index: Option<usize>,
    track: &[TrackSegment],
) -> String {
    let index = player_index.unwrap_or(0).min(room.players.len().saturating_sub(1));
    let player = &room.players[index];
    let finish = track.len().max(1);
    let segment_index = (player.distance.max(0) as usize).min(finish - 1);
    let segment = &track[segment_index];
    let player_hint = format!(
        "<p class=\"current-player\">Choosing for <strong>{}</strong></p>",
        html_escape(&player.name)
    );
    let boost_disabled = if player.energy < 3 {
        "disabled aria-disabled=\"true\" title=\"Piet Boost needs 3 energy\""
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
        player_hint = player_hint,
        endpoint = html_escape(endpoint),
        room = html_escape(&room.code),
        token = html_escape(token),
        segment_number = segment_index + 1,
        finish = finish,
        terrain = html_escape(&segment.terrain),
        terrain_name = html_escape(&capitalize(&segment.terrain)),
        symbol = terrain_symbol(&segment.terrain),
        energy = player.energy,
        boost_disabled = boost_disabled,
        piet_boost = piet_boost,
        boost_note = if player.energy < 3 {
            " · unavailable now"
        } else {
            ""
        },
    )
}

fn render_rematch_form(room: &Room, token: &str) -> String {
    format!(
        r#"<form method="post" action="/rematch" class="actions single-action">
<input type="hidden" name="room" value="{room}">
<input type="hidden" name="token" value="{token}">
<button type="submit"><strong>Start rematch</strong><span>Reset both racers and race again</span></button>
</form>"#,
        room = html_escape(&room.code),
        token = html_escape(token)
    )
}

fn render_track(track: &[TrackSegment]) -> String {
    let mut output = String::new();
    for (index, segment) in track.iter().enumerate() {
        let terrain = html_escape(&segment.terrain);
        let symbol = terrain_symbol(&segment.terrain);
        std::fmt::Write::write_fmt(
            &mut output,
            format_args!(
                "<span class=\"segment {terrain}\" title=\"Segment {} · {terrain}: speed {:+}, boost +{}, recovery +{}\"><b>{symbol}</b><small>{}</small></span>",
                index + 1,
                segment.speed,
                segment.boost,
                segment.recovery,
                index + 1
            ),
        )
        .expect("writing to a String cannot fail");
    }
    output
}

fn terrain_symbol(terrain: &str) -> &'static str {
    match terrain {
        "curve" => "◒",
        "mud" => "≈",
        "jump" => "▲",
        _ => "▰",
    }
}

fn track_fingerprint(track: &[TrackSegment]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for segment in track {
        for byte in segment.terrain.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash ^= (segment.speed as i64 as u64)
            ^ ((segment.boost as i64 as u64) << 8)
            ^ ((segment.recovery as i64 as u64) << 16);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:08X}", hash as u32)
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
