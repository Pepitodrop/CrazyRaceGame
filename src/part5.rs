fn page_shell(title: &str, extra_head: &str, content: &str) -> String {
    let piet_easter_egg = render_piet_easter_egg();
    format!(
        r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<meta name="theme-color" content="#0b1020">
<link rel="icon" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'%3E%3Crect width='64' height='64' rx='14' fill='%230b1020'/%3E%3Cpath d='M17 12v40' stroke='%23f5f7ff' stroke-width='6' stroke-linecap='round'/%3E%3Cpath d='M20 14h30v24H20z' fill='%236fe7ff'/%3E%3Cpath d='M20 14h10v8H20zm20 0h10v8H40zM30 22h10v8H30zM20 30h10v8H20zm20 0h10v8H40z' fill='%23b38cff'/%3E%3C/svg%3E">
<title>{title}</title>
{extra_head}
<style>
:root{{color-scheme:dark;--bg:#070b16;--panel:#11182b;--panel-2:#18223a;--text:#f5f7ff;--muted:#aeb9d5;--accent:#6fe7ff;--accent-2:#b38cff;--success:#79f2a6;--danger:#ff7b91;--border:rgba(255,255,255,.11)}}
*{{box-sizing:border-box}}
html{{-webkit-text-size-adjust:100%;background:var(--bg)}}
body{{margin:0;min-height:100vh;font-family:Inter,ui-sans-serif,system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;background:radial-gradient(circle at top,#19294d 0,#0b1020 38%,var(--bg) 80%);color:var(--text);line-height:1.55}}
main{{width:min(1120px,calc(100% - 32px));margin:0 auto;padding:48px 0 64px}}
h1,h2,h3,p{{margin-top:0}}h1{{font-size:clamp(2.2rem,7vw,5rem);line-height:.95;margin-bottom:18px;letter-spacing:-.045em}}h2{{font-size:1.3rem}}h3{{font-size:1.15rem;margin-bottom:14px}}
a{{color:var(--accent)}}code{{color:#dffaff;background:#090e1b;padding:.15rem .38rem;border-radius:.4rem}}
.hero{{padding:30px 0 38px;max-width:760px}}.hero p:not(.eyebrow){{font-size:clamp(1rem,2.4vw,1.3rem);color:var(--muted)}}
.eyebrow,.mode-tag{{font-size:.76rem;font-weight:800;text-transform:uppercase;letter-spacing:.16em;color:var(--accent);margin-bottom:10px}}
.grid{{display:grid;gap:18px}}.grid.two{{grid-template-columns:repeat(2,minmax(0,1fr))}}.grid.three{{grid-template-columns:repeat(3,minmax(0,1fr))}}
.card{{background:linear-gradient(145deg,rgba(24,34,58,.96),rgba(13,20,37,.96));border:1px solid var(--border);border-radius:20px;padding:22px;box-shadow:0 18px 50px rgba(0,0,0,.24)}}.local-card{{border-color:rgba(179,140,255,.42)}}
form{{display:grid;gap:14px}}label{{display:grid;gap:7px;color:var(--muted);font-weight:700;font-size:.9rem}}input{{width:100%;min-height:48px;border:1px solid var(--border);border-radius:12px;padding:12px 14px;background:#090e1b;color:var(--text);font:inherit;font-size:16px}}input:focus{{outline:3px solid rgba(111,231,255,.22);border-color:var(--accent)}}
button,.button-link{{min-height:50px;border:0;border-radius:13px;padding:13px 16px;background:linear-gradient(135deg,var(--accent),var(--accent-2));color:#07101c;font:inherit;font-weight:900;cursor:pointer;text-decoration:none;text-align:center;touch-action:manipulation}}button:hover,.button-link:hover{{filter:brightness(1.08)}}button:disabled{{cursor:not-allowed;filter:grayscale(.75);opacity:.48}}
.stack-note{{margin-top:18px}}.topbar{{display:flex;justify-content:space-between;align-items:flex-end;gap:18px;margin-bottom:20px}}.topbar h1{{font-size:clamp(2.2rem,10vw,4.4rem);margin:0;letter-spacing:.08em}}.round{{background:var(--panel-2);border:1px solid var(--border);border-radius:999px;padding:10px 15px;font-weight:800;white-space:nowrap}}
.racers{{margin-bottom:18px}}.racer{{position:relative;overflow:hidden}}.active-racer{{border-color:rgba(111,231,255,.55);box-shadow:0 0 0 2px rgba(111,231,255,.08),0 18px 50px rgba(0,0,0,.24)}}.muted{{opacity:.68}}.stats{{display:flex;justify-content:space-between;gap:12px;color:var(--muted);font-size:.9rem}}.stats strong{{color:var(--text)}}.status-labels{{display:flex;justify-content:space-between;gap:10px;margin-top:10px;color:var(--muted);font-size:.78rem}}
.progress,.energy-bar{{height:11px;margin-top:12px;background:#080d19;border-radius:999px;overflow:hidden;border:1px solid var(--border)}}.progress span{{display:block;height:100%;background:linear-gradient(90deg,var(--accent),var(--accent-2));border-radius:inherit}}.energy-bar{{height:7px;margin-top:8px}}.energy-bar span{{display:block;height:100%;background:linear-gradient(90deg,#79f2a6,#f4d35e);border-radius:inherit}}
.announcer,.track-card,.action-card{{margin-top:18px}}blockquote{{margin:0 0 14px;font-size:clamp(1.15rem,3vw,1.55rem);font-weight:800;line-height:1.35}}.announcer>p:last-child{{margin-bottom:0;color:var(--muted)}}
.track-heading{{display:flex;justify-content:space-between;align-items:flex-start;gap:20px}}.track-heading h2{{margin-bottom:5px}}.track-heading p,.track-help{{color:var(--muted);font-size:.82rem}}.track-help{{white-space:nowrap}}.track-scroll{{overflow-x:auto;padding:4px 2px 12px;overscroll-behavior-inline:contain}}.track{{display:flex;gap:7px;min-width:max-content}}.segment{{width:44px;height:48px;border-radius:10px;display:grid;place-items:center;font-size:.86rem;font-weight:900;border:1px solid rgba(255,255,255,.15);background:#28344f;line-height:1}}.segment b{{font-size:1rem}}.segment small{{font-size:.66rem;opacity:.85}}.segment.curve{{background:#59438d}}.segment.mud{{background:#7a5c36}}.segment.jump{{background:#1b716e}}.legend{{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:10px}}.legend-item{{display:flex;align-items:center;gap:9px;padding:9px 10px;border-radius:11px;background:#28344f;border:1px solid var(--border)}}.legend-item.curve{{background:#59438d}}.legend-item.mud{{background:#7a5c36}}.legend-item.jump{{background:#1b716e}}.legend-item b{{font-size:1.1rem}}.legend-item span{{display:grid}}.legend-item small{{font-size:.68rem;opacity:.8}}
.turn-status{{display:flex;flex-wrap:wrap;align-items:center;gap:9px 14px;margin-bottom:14px;padding:12px 14px;border-radius:12px;background:#090e1b;color:var(--muted);font-size:.84rem}}.terrain-pill{{padding:5px 9px;border-radius:999px;color:white;background:#28344f}}.terrain-pill.curve{{background:#59438d}}.terrain-pill.mud{{background:#7a5c36}}.terrain-pill.jump{{background:#1b716e}}.actions{{grid-template-columns:repeat(3,minmax(0,1fr))}}.actions button{{display:grid;gap:3px;text-align:left;align-content:start}}.actions button span{{font-weight:650;font-size:.78rem;opacity:.75}}.single-action{{grid-template-columns:1fr}}.notice,.handover{{padding:15px;border-radius:13px;background:rgba(111,231,255,.09);border:1px solid rgba(111,231,255,.22);color:#dffaff}}.notice{{margin-bottom:0}}.handover{{margin-bottom:14px}}.handover p,.current-player{{margin:0}}.current-player{{margin-bottom:12px;color:var(--muted)}}
.winner{{margin-bottom:18px;padding:16px 20px;border-radius:15px;background:rgba(121,242,166,.14);border:1px solid rgba(121,242,166,.36);color:var(--success);font-weight:900;font-size:1.05rem}}.footer-link{{text-align:center;margin:22px 0 0}}.error{{max-width:620px;margin:10vh auto}}.error .button-link{{display:inline-flex;align-items:center;justify-content:center;margin-top:8px}}
.piet-egg{{margin:44px auto 0;max-width:720px;color:var(--muted);text-align:center}}.piet-egg summary{{cursor:pointer;color:var(--accent);font-size:.8rem;letter-spacing:.08em;text-transform:uppercase}}.piet-egg figure{{margin:16px 0 0;padding:18px;border:1px dashed rgba(111,231,255,.3);border-radius:16px;background:rgba(9,14,27,.72)}}.piet-egg svg{{display:block;width:min(100%,360px);height:auto;margin:0 auto;image-rendering:pixelated}}.piet-egg figcaption{{margin-top:12px;font-size:.78rem}}
@media (max-width:820px){{main{{width:min(100% - 24px,680px);padding:28px 0 calc(44px + env(safe-area-inset-bottom))}}.grid.three,.grid.two{{grid-template-columns:1fr}}.hero{{padding-top:16px}}.topbar{{align-items:center}}.racers{{gap:12px}}.card{{padding:18px;border-radius:17px}}.actions,.legend{{grid-template-columns:1fr}}.actions button{{min-height:62px}}.track-heading{{display:block}}.track-help{{display:block;margin-bottom:8px}}}}
@media (max-width:480px){{main{{width:min(100% - 16px,460px)}}.topbar h1{{font-size:2.25rem}}.round{{padding:8px 11px;font-size:.86rem}}.stats,.status-labels{{font-size:.78rem}}.status-labels{{display:grid}}.segment{{width:40px;height:44px}}}}
@media (prefers-reduced-motion:no-preference){{button,.button-link{{transition:filter .15s ease,transform .15s ease}}button:active,.button-link:active{{transform:scale(.985)}}}}
</style>
</head>
<body><main>{content}{piet_easter_egg}</main></body>
</html>"##,
        title = html_escape(title),
        extra_head = extra_head,
        content = content,
        piet_easter_egg = piet_easter_egg,
    )
}

fn render_piet_easter_egg() -> String {
    let source = match fs::read_to_string(PIET_PROGRAM) {
        Ok(source) => source,
        Err(_) => return String::new(),
    };
    let mut tokens = Vec::new();
    for line in source.lines() {
        let clean = line.split('#').next().unwrap_or("");
        tokens.extend(clean.split_whitespace());
    }
    let mut iter = tokens.into_iter();
    if iter.next() != Some("P3") {
        return String::new();
    }
    let width = iter
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    let height = iter
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    let max_value = iter
        .next()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(0);
    if width == 0 || height == 0 || max_value != 255 {
        return String::new();
    }

    let codel_size = 36usize;
    let mut rectangles = String::new();
    for index in 0..width.saturating_mul(height) {
        let red = iter
            .next()
            .and_then(|value| value.parse::<u8>().ok())
            .unwrap_or(0);
        let green = iter
            .next()
            .and_then(|value| value.parse::<u8>().ok())
            .unwrap_or(0);
        let blue = iter
            .next()
            .and_then(|value| value.parse::<u8>().ok())
            .unwrap_or(0);
        let x = (index % width) * codel_size;
        let y = (index / width) * codel_size;
        std::fmt::Write::write_fmt(
            &mut rectangles,
            format_args!(
                "<rect x=\"{x}\" y=\"{y}\" width=\"{codel_size}\" height=\"{codel_size}\" fill=\"rgb({red},{green},{blue})\"/>"
            ),
        )
        .expect("writing to a String cannot fail");
    }

    format!(
        r#"<details class="piet-egg"><summary>Piet easter egg</summary><figure><svg role="img" aria-label="Magnified Piet boost oracle program" viewBox="0 0 {svg_width} {svg_height}" xmlns="http://www.w3.org/2000/svg" shape-rendering="crispEdges">{rectangles}</svg><figcaption>This is the actual adaptive image program from <code>piet/boost_oracle.ppm</code>. It reads round, terrain, comeback gap, and energy, then computes <code>1 + (sum mod 4)</code> for a changing +1 to +4 boost.</figcaption></figure></details>"#,
        svg_width = width * codel_size,
        svg_height = height * codel_size,
        rectangles = rectangles,
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

    fn segment(terrain: &str) -> TrackSegment {
        TrackSegment {
            terrain: terrain.to_string(),
            speed: 0,
            boost: 1,
            recovery: 2,
        }
    }

    fn test_app() -> AppState {
        AppState {
            rooms: HashMap::new(),
            track: vec![segment("straight"); 8],
            piet_boost: 3,
            announcements: vec!["Test announcement.".to_string()],
            entropy: 42,
            max_rooms: 10,
            room_ttl_seconds: 7_200,
        }
    }

    #[test]
    fn drift_rewards_curves() {
        let straight = movement_for(RaceAction::Drift, 5, &segment("straight"), 3);
        let curve = movement_for(RaceAction::Drift, 5, &segment("curve"), 3);
        assert!(curve.0 > straight.0);
        assert_eq!(curve.1, 2);
    }

    #[test]
    fn boost_needs_energy() {
        assert_eq!(movement_for(RaceAction::Boost, 2, &segment("jump"), 3).0, 1);
        assert!(movement_for(RaceAction::Boost, 3, &segment("jump"), 3).0 > 1);
    }

    #[test]
    fn piet_oracle_outputs_three() {
        let source = "P3\n5 1\n255\n255 192 192 255 192 192 255 192 192 255 0 0 192 0 192\n";
        assert_eq!(run_piet_source(source).unwrap(), 3);
    }

    #[test]
    fn adaptive_piet_oracle_changes_with_inputs_and_stays_bounded() {
        let source = include_str!("../piet/boost_oracle.ppm");
        assert_eq!(
            run_piet_source_with_inputs(source, &[1, 0, 0, 1]).unwrap(),
            3
        );
        assert_eq!(
            run_piet_source_with_inputs(source, &[2, 3, 1, 2]).unwrap(),
            1
        );
        for round in 1..12 {
            let value = run_piet_source_with_inputs(source, &[round, 2, 3, 1]).unwrap();
            assert!((1..=4).contains(&value));
        }
    }

    #[test]
    fn mobile_viewport_favicon_and_easter_egg_are_present() {
        let page = render_landing_page();
        assert!(page.contains("width=device-width"));
        assert!(page.contains("Local 1v1"));
        let shell = page_shell("Test", "", &page);
        assert!(shell.contains("rel=\"icon\""));
        assert!(shell.contains("Piet easter egg"));
    }

    #[test]
    fn track_uses_visible_symbols_and_fingerprint() {
        let track = vec![
            segment("straight"),
            segment("curve"),
            segment("mud"),
            segment("jump"),
        ];
        let html = render_track(&track);
        assert!(html.contains("▰"));
        assert!(html.contains("◒"));
        assert!(html.contains("≈"));
        assert!(html.contains("▲"));
        assert_eq!(track_fingerprint(&track).len(), 8);
    }

    #[test]
    fn racer_display_clamps_finish_counter_and_shows_energy() {
        let player = Player {
            name: "Test".to_string(),
            token: "token".to_string(),
            distance: 25,
            energy: 4,
            submitted: None,
        };
        let html = render_racer(&player, 20, true);
        assert!(html.contains("20</strong> / 20"));
        assert!(html.contains("Finished · score 25"));
        assert!(html.contains("4</strong> / 10 energy"));
    }

    #[test]
    fn pending_online_move_does_not_reveal_action() {
        let summary = move_locked_summary("Alice");
        assert_eq!(summary, "Alice locked in a move. Waiting for the rival.");
        assert!(!summary.contains("Accelerate"));
        assert!(!summary.contains("Drift"));
        assert!(!summary.contains("Boost"));
    }

    #[test]
    fn access_tokens_are_256_bit_hex_strings() {
        let mut app = test_app();
        let first = next_token(&mut app);
        let second = next_token(&mut app);
        assert_eq!(first.len(), 64);
        assert_eq!(second.len(), 64);
        assert!(first.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert!(second.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert_ne!(first, second);
    }

    #[test]
    fn room_codes_are_strictly_validated() {
        assert_eq!(clean_room_code(" abc234 ").unwrap(), "ABC234");
        assert!(clean_room_code("ABC").is_err());
        assert!(clean_room_code("ABC10O").is_err());
        assert!(clean_room_code("ABC-23").is_err());
    }

    #[test]
    fn security_headers_disable_caching_and_embedding() {
        assert!(SECURITY_HEADERS.contains("Cache-Control: no-store"));
        assert!(SECURITY_HEADERS.contains("Content-Security-Policy:"));
        assert!(SECURITY_HEADERS.contains("frame-ancestors 'none'"));
        assert!(SECURITY_HEADERS.contains("Referrer-Policy: no-referrer"));
    }
}
