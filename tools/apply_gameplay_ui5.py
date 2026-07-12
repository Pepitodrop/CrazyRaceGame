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


PART5 = r'''fn page_shell(title: &str, extra_head: &str, content: &str) -> String {
    format!(
        r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<meta name="theme-color" content="#0b1020">
<link rel="icon" type="image/svg+xml" href="/favicon.svg">
<title>{title}</title>
{extra_head}
<style>
:root{{color-scheme:dark;--bg:#070b16;--panel:#11182b;--panel-2:#18223a;--text:#f5f7ff;--muted:#aeb9d5;--accent:#6fe7ff;--accent-2:#b38cff;--success:#79f2a6;--danger:#ff7b91;--border:rgba(255,255,255,.11)}}
*{{box-sizing:border-box}}
html{{-webkit-text-size-adjust:100%;background:var(--bg)}}
body{{margin:0;min-height:100vh;font-family:Inter,ui-sans-serif,system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;background:radial-gradient(circle at top,#19294d 0,#0b1020 38%,var(--bg) 80%);color:var(--text);line-height:1.55}}
main{{width:min(1120px,calc(100% - 32px));margin:0 auto;padding:48px 0 64px}}
h1,h2,h3,p{{margin-top:0}}h1{{font-size:clamp(2.2rem,7vw,5rem);line-height:.95;margin-bottom:18px;letter-spacing:-.045em}}h2{{font-size:1.3rem}}h3{{font-size:1.15rem;margin-bottom:14px}}
a{{color:var(--accent)}}
.hero{{padding:30px 0 38px;max-width:760px}}.hero p:not(.eyebrow){{font-size:clamp(1rem,2.4vw,1.3rem);color:var(--muted)}}
.eyebrow,.mode-tag{{font-size:.76rem;font-weight:800;text-transform:uppercase;letter-spacing:.16em;color:var(--accent);margin-bottom:10px}}
.grid{{display:grid;gap:18px}}.grid.two{{grid-template-columns:repeat(2,minmax(0,1fr))}}.grid.three{{grid-template-columns:repeat(3,minmax(0,1fr))}}
.card{{background:linear-gradient(145deg,rgba(24,34,58,.96),rgba(13,20,37,.96));border:1px solid var(--border);border-radius:20px;padding:22px;box-shadow:0 18px 50px rgba(0,0,0,.24)}}.local-card{{border-color:rgba(179,140,255,.42)}}
form{{display:grid;gap:14px}}label{{display:grid;gap:7px;color:var(--muted);font-weight:700;font-size:.9rem}}input{{width:100%;min-height:48px;border:1px solid var(--border);border-radius:12px;padding:12px 14px;background:#090e1b;color:var(--text);font:inherit;font-size:16px}}input:focus{{outline:3px solid rgba(111,231,255,.22);border-color:var(--accent)}}
button,.button-link{{min-height:50px;border:0;border-radius:13px;padding:13px 16px;background:linear-gradient(135deg,var(--accent),var(--accent-2));color:#07101c;font:inherit;font-weight:900;cursor:pointer;text-decoration:none;text-align:center;touch-action:manipulation}}button:hover,.button-link:hover{{filter:brightness(1.08)}}button:disabled{{cursor:not-allowed;filter:grayscale(.75);opacity:.48;transform:none}}
.stack-note{{margin-top:18px}}.topbar{{display:flex;justify-content:space-between;align-items:flex-end;gap:18px;margin-bottom:20px}}.topbar h1{{font-size:clamp(2.2rem,10vw,4.4rem);margin:0;letter-spacing:.08em}}.round{{background:var(--panel-2);border:1px solid var(--border);border-radius:999px;padding:10px 15px;font-weight:800;white-space:nowrap}}
.racers{{margin-bottom:18px}}.racer{{position:relative;overflow:hidden}}.active-racer{{border-color:rgba(111,231,255,.55);box-shadow:0 0 0 2px rgba(111,231,255,.08),0 18px 50px rgba(0,0,0,.24)}}.muted{{opacity:.68}}.racer-metrics{{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px}}.metric{{display:grid;gap:2px;padding:10px 12px;background:#0b1120;border:1px solid var(--border);border-radius:11px}}.metric span,.energy-row span{{color:var(--muted);font-size:.78rem}}.metric strong{{font-size:.92rem}}
.progress,.energy-meter{{height:11px;margin-top:13px;background:#080d19;border-radius:999px;overflow:hidden;border:1px solid var(--border)}}.progress span{{display:block;height:100%;background:linear-gradient(90deg,var(--accent),var(--accent-2));border-radius:inherit}}.energy-row{{display:flex;justify-content:space-between;align-items:center;margin-top:13px}}.energy-meter{{margin-top:6px}}.energy-meter span{{display:block;height:100%;background:linear-gradient(90deg,#79f2a6,#f2d479);border-radius:inherit}}
.announcer,.track-card,.action-card,.piet-easter-egg{{margin-top:18px}}blockquote{{margin:0 0 14px;font-size:clamp(1.15rem,3vw,1.55rem);font-weight:800;line-height:1.35}}.announcer>p:last-child,.section-help{{margin-bottom:0;color:var(--muted)}}
.track-scroll{{overflow-x:auto;padding:15px 2px 12px;overscroll-behavior-inline:contain}}.track{{display:flex;gap:7px;min-width:max-content}}.segment{{position:relative;width:44px;height:52px;border-radius:10px;display:grid;place-items:center;font-weight:900;border:1px solid rgba(255,255,255,.15);background:#28344f}}.segment.curve{{background:#59438d}}.segment.mud{{background:#7a5c36}}.segment.jump{{background:#1b716e}}.terrain-symbol{{font-size:1.15rem;line-height:1;margin-top:4px}}.segment-number{{font-size:.68rem;opacity:.85}}.player-markers{{position:absolute;top:-12px;left:2px;right:2px;display:flex;justify-content:center;gap:2px}}.player-marker{{padding:1px 4px;border-radius:999px;font-size:.57rem;color:#07101c;background:var(--accent)}}.marker-2{{background:var(--accent-2)}}
.legend{{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:9px;margin-top:5px}}.legend>span{{display:flex;align-items:center;gap:8px;padding:8px 9px;border:1px solid var(--border);border-radius:11px;background:#0b1120}}.legend-swatch{{width:30px;height:30px;display:grid;place-items:center;border-radius:8px;font-style:normal;font-weight:900;background:#28344f}}.legend-swatch.curve{{background:#59438d}}.legend-swatch.mud{{background:#7a5c36}}.legend-swatch.jump{{background:#1b716e}}.legend strong,.legend small{{display:block}}.legend small{{color:var(--muted);font-size:.68rem}}
.actions{{grid-template-columns:repeat(3,minmax(0,1fr))}}.actions button{{display:grid;gap:3px;text-align:left;align-content:start}}.actions button span{{font-weight:650;font-size:.78rem;opacity:.75}}.single-action{{grid-template-columns:1fr}}.notice,.handover{{padding:15px;border-radius:13px;background:rgba(111,231,255,.09);border:1px solid rgba(111,231,255,.22);color:#dffaff}}.notice{{margin-bottom:0}}.handover{{margin-bottom:14px}}.handover p,.current-player{{margin:0}}.current-player{{margin-bottom:8px;color:var(--muted)}}.energy-help{{margin:0 0 12px;color:var(--muted);font-size:.88rem}}
.winner{{margin-bottom:18px;padding:16px 20px;border-radius:15px;background:rgba(121,242,166,.14);border:1px solid rgba(121,242,166,.36);color:var(--success);font-weight:900;font-size:1.05rem}}.footer-link{{text-align:center;margin:22px 0 0}}.error{{max-width:620px;margin:10vh auto}}.error .button-link{{display:inline-flex;align-items:center;justify-content:center;margin-top:8px}}
.piet-easter-egg summary{{cursor:pointer;font-weight:850;color:var(--accent)}}.piet-easter-egg p{{margin:14px 0;color:var(--muted)}}.piet-painting{{display:block;width:min(100%,500px);height:auto;border:1px solid var(--border);border-radius:10px;background:white}}.piet-easter-egg code{{display:inline-block;margin-top:12px;padding:6px 9px;border-radius:8px;background:#080d19;color:var(--text)}}
@media (max-width:820px){{main{{width:min(100% - 24px,680px);padding:28px 0 calc(44px + env(safe-area-inset-bottom))}}.grid.three,.grid.two{{grid-template-columns:1fr}}.hero{{padding-top:16px}}.topbar{{align-items:center}}.racers{{gap:12px}}.card{{padding:18px;border-radius:17px}}.actions{{grid-template-columns:1fr}}.actions button{{min-height:62px}}.legend{{grid-template-columns:repeat(2,minmax(0,1fr))}}}}
@media (max-width:480px){{main{{width:min(100% - 16px,460px)}}.topbar h1{{font-size:2.25rem}}.round{{padding:8px 11px;font-size:.86rem}}.racer-metrics{{grid-template-columns:1fr}}.segment{{width:40px;height:49px}}.legend{{grid-template-columns:1fr}}}}
@media (prefers-reduced-motion:no-preference){{button,.button-link{{transition:filter .15s ease,transform .15s ease}}button:active,.button-link:active{{transform:scale(.985)}}}}
</style>
</head>
<body><main>{content}</main></body>
</html>"##,
        title = html_escape(title),
        extra_head = extra_head,
        content = content
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
    fn energy_validation_blocks_unaffordable_actions() {
        assert!(validate_action_energy(RaceAction::Accelerate, 0).is_err());
        assert!(validate_action_energy(RaceAction::Boost, 2).is_err());
        assert!(validate_action_energy(RaceAction::Drift, 0).is_ok());
        assert!(validate_action_energy(RaceAction::Boost, 3).is_ok());
    }

    #[test]
    fn room_tracks_vary_but_keep_start_and_finish() {
        let base = vec![
            segment("straight"),
            segment("curve"),
            segment("mud"),
            segment("jump"),
            segment("curve"),
            segment("mud"),
            segment("jump"),
            segment("straight"),
        ];
        let first = track_for_room(&base, 11);
        let second = track_for_room(&base, 97);
        assert_eq!(first.first().unwrap().terrain, "straight");
        assert_eq!(first.last().unwrap().terrain, "straight");
        assert_eq!(second.first().unwrap().terrain, "straight");
        assert_eq!(second.last().unwrap().terrain, "straight");
        assert_ne!(track_signature(&first), track_signature(&second));
    }

    #[test]
    fn piet_oracle_outputs_three() {
        let source = "P3\n5 1\n255\n255 192 192 255 192 192 255 192 192 255 0 0 192 0 192\n";
        assert_eq!(run_piet_source(source).unwrap(), 3);
    }

    #[test]
    fn mobile_viewport_favicon_and_easter_egg_are_present() {
        let page = render_landing_page();
        assert!(page.contains("width=device-width"));
        assert!(page.contains("/favicon.svg"));
        assert!(page.contains("Local 1v1"));
        assert!(render_piet_easter_egg().contains("Piet boost program"));
    }

    #[test]
    fn racer_display_caps_progress_at_finish() {
        let mut player = new_player("Test".to_string(), "token".to_string());
        let track = vec![segment("straight"); 8];
        player.distance = 12;
        let html = render_racer(&player, &track, true);
        assert!(html.contains("Segment 8 of 8"));
        assert!(!html.contains("Segment 12 of 8"));
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
        assert!(SECURITY_HEADERS.contains("img-src 'self' data:"));
        assert!(SECURITY_HEADERS.contains("frame-ancestors 'none'"));
        assert!(SECURITY_HEADERS.contains("Referrer-Policy: no-referrer"));
    }
}
'''
Path("src/part5.rs").write_text(PART5)
