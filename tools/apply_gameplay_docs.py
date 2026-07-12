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


replace_once(
    "CHANGELOG.md",
    '''## [Unreleased]\n\nNo unreleased changes.\n\n## [1.0.1] - 2026-07-12''',
    '''## [Unreleased]\n\nNo unreleased changes.\n\n## [1.1.0] - 2026-07-12\n\n### Added\n\n- Added a proper SVG browser-tab icon.\n- Added a collapsible Piet source-painting easter egg to every race page.\n- Added visible terrain symbols, color-matched legend cards, and P1/P2 position markers.\n- Added explicit segment, current-terrain, and energy meters to racer cards.\n\n### Changed\n\n- Every room now receives its own deterministic shuffle of the R-generated circuit; rematches shuffle again.\n- Unaffordable Accelerate and Piet Boost actions are disabled in the UI and rejected by the server.\n- Finish progress is capped at the actual segment count while overshoot remains internal to same-round winner resolution.\n- The default Compose profile is compatible with Snap Docker; the production profile retains `no-new-privileges`.\n\n### Fixed\n\n- Clarified terrain colors and symbols that were previously too small to read reliably.\n- Removed confusing values such as `25 / 20` from the racer display.\n- Added CI coverage for favicon delivery, circuit variation, energy validation, terrain legend rendering, and the Piet easter egg.\n\n## [1.0.1] - 2026-07-12''',
)
replace_once(
    "CHANGELOG.md",
    '''[Unreleased]: https://github.com/Pepitodrop/CrazyRaceGame/compare/v1.0.1...HEAD\n[1.0.1]:''',
    '''[Unreleased]: https://github.com/Pepitodrop/CrazyRaceGame/compare/v1.1.0...HEAD\n[1.1.0]: https://github.com/Pepitodrop/CrazyRaceGame/compare/v1.0.1...v1.1.0\n[1.0.1]:''',
)

Path("docs/RELEASE_NOTES_v1.1.0.md").write_text('''# Crazy Race 1.1.0\n\nCrazy Race 1.1.0 is a gameplay-clarity and compatibility release.\n\n## Highlights\n\n- Every new room receives a different deterministic shuffle of the R-generated circuit.\n- Terrain tiles now show readable symbols, matching legend colors, and P1/P2 position markers.\n- Racer cards show explicit segment progress, current terrain, and an energy meter.\n- Unaffordable actions are disabled and also rejected server-side.\n- The browser tab now has a Crazy Race SVG icon.\n- The real five-codel Piet boost painting is available as a collapsible easter egg.\n- The local Docker Compose profile works with Snap Docker's AppArmor restrictions, while the production profile retains stronger `no-new-privileges` hardening.\n\n## Upgrade\n\n```bash\ndocker compose down --remove-orphans\ngit pull --ff-only\ndocker compose up --build\n```\n\nOpen `http://localhost:8080`.\n\n## Production boundary\n\nThe application remains intended for a single-instance hobby, demonstration, or small-community deployment behind HTTPS. Active rooms remain in memory and are cleared when the process restarts.\n''')

readme = Path("README.md")
text = readme.read_text()
text = text.replace("release-v1.0.1-blue.svg", "release-v1.1.0-blue.svg")
text = text.replace("Crazy Race 1.0.1 is", "Crazy Race 1.1.0 is")
text = text.replace(
    "**R**: deterministic procedural track generation at container startup.",
    "**R**: deterministic procedural track-data generation; Rust gives each room its own shuffle.",
)
text = text.replace(
    "Change the deterministic circuit seed with:",
    "Change the deterministic base circuit seed with (rooms still receive separate shuffles):",
)
text = text.replace(
    '''When updating from 1.0.0, remove the old restart-loop container before starting 1.0.1:\n\n```bash\ndocker compose down\ngit pull --ff-only\ndocker compose up --build\n```''',
    '''When updating, recreate the container so the current image and Compose settings are used:\n\n```bash\ndocker compose down --remove-orphans\ngit pull --ff-only\ndocker compose up --build\n```\n\nThe default Compose profile intentionally works with the Snap-packaged Docker Engine on Ubuntu. The hardened production profile retains `no-new-privileges` and should be run with Docker Engine from Docker's official packages or Docker Desktop.''',
)
new_gameplay = '''## How to play\n\n### Objective and racer cards\n\nBoth racers start at **Segment 0 of 20** with **5 / 10 energy**. Reach Segment 20 first. The racer cards now label all values explicitly:\n\n- **Progress** — the completed segment count, capped at the finish;\n- **Current terrain** — the tile used to resolve the next action;\n- **Energy** — the current value out of 10, with a separate energy bar;\n- the main progress bar — percentage of the circuit completed.\n\n### What happens in one round\n\n1. Both racers choose exactly one available action.\n2. Each action uses the terrain at that racer's current segment.\n3. Both moves resolve together.\n4. Distance and energy are updated.\n5. The announcer reports the completed round.\n6. If nobody has finished, the next round begins.\n\nThe badge at the top shows the round you are **about to play**. The announcer text normally describes the round that just finished.\n\n### Local pass-and-play\n\n1. Player 1 chooses an action.\n2. Pass the device to Player 2.\n3. Player 1's action remains hidden.\n4. Player 2 chooses.\n5. The round resolves and the next round returns to Player 1.\n\n### Reading the circuit\n\nEvery room receives its own deterministic shuffle of the R-generated track data. A rematch shuffles again. The tiles display their segment number, terrain symbol, terrain color, and current player markers:\n\n| Color | Symbol | Terrain | Main effect |\n| --- | --- | --- | --- |\n| Blue-gray | `→` | Straight | Usually the fastest general terrain |\n| Purple | `↪` | Curve | Gives Drift its largest bonus |\n| Amber/brown | `≈` | Mud | Usually reduces speed |\n| Teal | `▲` | Jump | Usually improves Piet Boost |\n\n`P1` and `P2` badges show the racers' current segments. Hovering a tile on desktop reveals its exact speed, boost, and recovery modifiers.\n\n### Actions and energy\n\n#### Accelerate\n\n- Costs **1 energy**.\n- Uses the current segment's speed modifier.\n- Disabled when energy is 0.\n\n#### Drift\n\n- Always available.\n- Strongest on curves and receives a smaller mud bonus.\n- Restores the current segment's recovery amount, up to 10 energy.\n- Use Drift whenever another action is disabled.\n\n#### Piet Boost\n\n- Requires and costs **3 energy**.\n- Adds the number emitted by the Piet painting (currently 3), plus segment speed and boost values.\n- Disabled when energy is below 3.\n\nThe server repeats these checks, so manually forged invalid action requests are rejected.\n\n### Winning and photo finishes\n\nThe UI stops at **Segment 20 of 20** instead of displaying confusing values such as `25 / 20`. Internally, same-round overshoot is still used to determine who crossed farther. If both raw finish distances are identical, the deterministic photo-finish rule selects the winner and the announcer result states that it was a photo finish.\n\n### Piet easter egg\n\nOpen **Reveal the Piet boost program** at the bottom of any race page. The magnified five-codel painting is the real executable Piet source: three light-red codels push the number 3, then red-to-dark-magenta outputs it.\n\n'''
text, count = re.subn(r"## How to play\n.*?## Production deployment\n", new_gameplay + "## Production deployment\n", text, count=1, flags=re.S)
if count != 1:
    raise SystemExit("could not replace README gameplay section")
text = text.replace(
    "- all Linux capabilities removed and `no-new-privileges` enabled;",
    "- all Linux capabilities removed; the production profile also enables `no-new-privileges`;",
)
text = text.replace(
    "`scripts/generate_track.R` produces a tab-separated track with terrain, speed, boost, and energy-recovery attributes. Rust launches it once when the container starts. `data/default_track.tsv` is used only if R generation fails.",
    "`scripts/generate_track.R` produces tab-separated terrain, speed, boost, and energy-recovery data. Rust launches it once when the container starts, then deterministically shuffles the middle segments for each room while preserving the R-generated values and straight start/finish. `data/default_track.tsv` is used only if R generation fails.",
)
text = text.replace("Crazy Race 1.0.1 is suitable", "Crazy Race 1.1.0 is suitable")
readme.write_text(text)

production = Path("docs/PRODUCTION.md")
text = production.read_text()
needle = "## 1. Configure\n"
if needle not in text:
    raise SystemExit("missing production configure heading")
text = text.replace(
    needle,
    '''## Docker package compatibility\n\nUbuntu's Snap-packaged Docker applies an AppArmor policy that can reject `no-new-privileges` container startup. The default `docker-compose.yml` omits that one flag so local development works with Snap Docker while retaining the non-root user, read-only filesystem, dropped capabilities, and resource limits.\n\nFor public production deployment, use Docker Engine from Docker's official packages or Docker Desktop and launch `compose.production.yml`, which retains `no-new-privileges`.\n\n## 1. Configure\n''',
    1,
)
production.write_text(text)

for path in [
    "tools/apply_gameplay_core.py",
    "tools/apply_gameplay_ui4.py",
    "tools/apply_gameplay_ui5.py",
    "tools/apply_gameplay_ops.py",
    "tools/apply_gameplay_docs.py",
    ".github/workflows/apply-gameplay-v110.yml",
]:
    Path(path).unlink(missing_ok=True)
