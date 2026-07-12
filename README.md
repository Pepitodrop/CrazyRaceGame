# Crazy Race Game

[![CI](https://github.com/Pepitodrop/CrazyRaceGame/actions/workflows/ci.yml/badge.svg)](https://github.com/Pepitodrop/CrazyRaceGame/actions/workflows/ci.yml)
[![Release](https://img.shields.io/badge/release-v1.1.0-blue.svg)](CHANGELOG.md)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

Crazy Race 1.1.0 is a Dockerized, browser-based **1v1 turn-based racing game** built around four deliberately unusual languages:

- **Rust**: dependency-free HTTP server, room management, game rules, responsive server-rendered UI, security controls, and the Piet interpreter.
- **R**: deterministic procedural track-data generation; Rust gives each room its own shuffle.
- **TrumpScript**: race announcements, executed by the original archived interpreter.
- **Piet**: an image-based boost oracle stored as an ASCII PPM painting.

The browser UI uses only server-rendered HTML and CSS. There is no JavaScript application and no separate frontend service.

## Quick start

You need Docker Engine or Docker Desktop with Docker Compose support.

```bash
git clone https://github.com/Pepitodrop/CrazyRaceGame.git
cd CrazyRaceGame
docker compose up --build
```

Open `http://localhost:8080` on a computer, phone, or tablet. Another device on the same Wi-Fi network can open the Docker host's local address, for example `http://192.168.1.20:8080`.

Change the deterministic base circuit seed with (rooms still receive separate shuffles):

```bash
TRACK_SEED=2026 docker compose up --build
```

When updating, recreate the container so the current image and Compose settings are used:

```bash
docker compose down --remove-orphans
git pull --ff-only
docker compose up --build
```

The default Compose profile intentionally works with the Snap-packaged Docker Engine on Ubuntu. The hardened production profile retains `no-new-privileges` and should be run with Docker Engine from Docker's official packages or Docker Desktop.

## How to play

### Objective and racer cards

Both racers start at **Segment 0 of 20** with **5 / 10 energy**. Reach Segment 20 first. The racer cards now label all values explicitly:

- **Progress** — the completed segment count, capped at the finish;
- **Current terrain** — the tile used to resolve the next action;
- **Energy** — the current value out of 10, with a separate energy bar;
- the main progress bar — percentage of the circuit completed.

### What happens in one round

1. Both racers choose exactly one available action.
2. Each action uses the terrain at that racer's current segment.
3. Both moves resolve together.
4. Distance and energy are updated.
5. The announcer reports the completed round.
6. If nobody has finished, the next round begins.

The badge at the top shows the round you are **about to play**. The announcer text normally describes the round that just finished.

### Local pass-and-play

1. Player 1 chooses an action.
2. Pass the device to Player 2.
3. Player 1's action remains hidden.
4. Player 2 chooses.
5. The round resolves and the next round returns to Player 1.

### Reading the circuit

Every room receives its own deterministic shuffle of the R-generated track data. A rematch shuffles again. The tiles display their segment number, terrain symbol, terrain color, and current player markers:

| Color | Symbol | Terrain | Main effect |
| --- | --- | --- | --- |
| Blue-gray | `→` | Straight | Usually the fastest general terrain |
| Purple | `↪` | Curve | Gives Drift its largest bonus |
| Amber/brown | `≈` | Mud | Usually reduces speed |
| Teal | `▲` | Jump | Usually improves Piet Boost |

`P1` and `P2` badges show the racers' current segments. Hovering a tile on desktop reveals its exact speed, boost, and recovery modifiers.

### Actions and energy

#### Accelerate

- Costs **1 energy**.
- Uses the current segment's speed modifier.
- Disabled when energy is 0.

#### Drift

- Always available.
- Strongest on curves and receives a smaller mud bonus.
- Restores the current segment's recovery amount, up to 10 energy.
- Use Drift whenever another action is disabled.

#### Piet Boost

- Requires and costs **3 energy**.
- Adds the number emitted by the Piet painting (currently 3), plus segment speed and boost values.
- Disabled when energy is below 3.

The server repeats these checks, so manually forged invalid action requests are rejected.

### Winning and photo finishes

The UI stops at **Segment 20 of 20** instead of displaying confusing values such as `25 / 20`. Internally, same-round overshoot is still used to determine who crossed farther. If both raw finish distances are identical, the deterministic photo-finish rule selects the winner and the announcer result states that it was a photo finish.

### Piet easter egg

Open **Reveal the Piet boost program** at the bottom of any race page. The magnified five-codel painting is the real executable Piet source: three light-red codels push the number 3, then red-to-dark-magenta outputs it.

## Production deployment

The hardened production profile binds the application to loopback so it can sit behind a TLS reverse proxy:

```bash
cp .env.example .env
docker compose -f compose.production.yml up -d --build
```

Use `deploy/Caddyfile.example` as a TLS reverse-proxy starting point. Full deployment, update, scaling, and operational instructions are in [`docs/PRODUCTION.md`](docs/PRODUCTION.md).

Production safeguards include:

- a non-root UID/GID;
- a read-only root filesystem and small writable `/tmp` tmpfs;
- all Linux capabilities removed; the production profile also enables `no-new-privileges`;
- CPU, memory, process, and log limits;
- cryptographically random 256-bit bearer tokens on Linux;
- request-header, request-body, URI, connection, and room limits;
- automatic expiry of inactive rooms;
- CSP, cache-control, referrer, framing, permissions, and cross-origin response headers;
- an HTTP-aware readiness check;
- end-to-end CI tests against the actual local and production Compose profiles.

## Playing modes

### Online or LAN room

One player creates a room and shares the six-character code. The second player joins from another browser window or another device. Both players lock in a move without seeing the rival's choice; the round resolves only after both submit.

### Local pass-and-play

Choose **Local 1v1**, enter two racer names, and play on one computer, phone, or tablet. Player 1 chooses a move, then passes the device to Player 2. The first move remains hidden until Player 2 submits.

## Mobile support

The same server-rendered pages adapt to narrow screens:

- racer panels stack vertically;
- action buttons become large full-width touch targets;
- the circuit scrolls horizontally;
- text and spacing scale down on small phones;
- safe-area padding supports modern mobile browsers.

No mobile application installation is required.

## Race rules reference

Each round, both racers choose one move:

- **Accelerate**: reliable speed, costs one energy.
- **Drift**: strongest on curves and restores energy according to the R-generated segment.
- **Piet Boost**: consumes three energy and adds the number emitted by the Piet painting.

The first racer across the final segment wins. If both cross in the same round, overshoot distance decides; an exact tie uses a deterministic room tie-breaker.

## Language architecture

### Rust

The Rust application uses only the standard library. It owns mutable game state in memory, renders every page on the server, validates room and local-game tokens, resolves turns, enforces resource limits, and interprets the Piet oracle.

The source is split into `src/part1.rs` through `src/part5.rs` and assembled by `src/main.rs` with `include!`, producing one binary.

### R

`scripts/generate_track.R` produces tab-separated terrain, speed, boost, and energy-recovery data. Rust launches it once when the container starts, then deterministically shuffles the middle segments for each room while preserving the R-generated values and straight start/finish. `data/default_track.tsv` is used only if R generation fails.

### TrumpScript and the limited Python exception

The Docker build fetches the original archived repository at the pinned commit:

```text
samshadwell/TrumpScript@3793b905925b55c0296b066586c7d612c7220ca0
```

The announcer source is `announcer/race.tr`, and Rust executes it through the original `TRUMP --shut-up` entrypoint. Python is installed **only** because the upstream TrumpScript interpreter is implemented in Python. No game logic, networking, rendering, track generation, security control, or Piet execution is written in Python.

A compatibility-only AST shim maps the interpreter's removed legacy `Module`, `Num`, `Str`, and `NameConstant` constructors to modern Python equivalents. The TrumpScript grammar and runtime behavior remain upstream code.

### Piet

`piet/boost_oracle.ppm` is executable source code as an image. Its codel transitions perform:

1. `push 3`
2. `out(number)`

Rust reads the PPM, interprets those Piet transitions, and uses the emitted `3` as the boost modifier.

## Validation

```bash
rustfmt --edition 2021 src/*.rs
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --release --locked
docker compose config --quiet
docker compose -f compose.production.yml config --quiet
docker compose up -d --build
bash scripts/smoke-test.sh http://127.0.0.1:8080
docker compose down
```

GitHub Actions additionally verifies image metadata and the non-root image user, executes the original pinned TrumpScript interpreter, and launches both Compose profiles under their hardened settings.

## Support boundary

Crazy Race 1.1.0 is suitable for public release and production deployment as a **single-instance hobby, demo, or small-community service** behind HTTPS. It is not an internet-scale multi-tenant platform.

Active rooms are stored in memory and disappear when the process restarts. Do not deploy multiple replicas without first moving room state to a shared transactional store.

Player tokens are bearer credentials contained in game URLs. Public deployments must use HTTPS, and the application sends `Referrer-Policy: no-referrer` plus `Cache-Control: no-store` to reduce leakage and caching risk.

## Project documentation

- [`CHANGELOG.md`](CHANGELOG.md) — version history and known limitations
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — development and pull-request guidance
- [`SECURITY.md`](SECURITY.md) — private vulnerability reporting
- [`docs/PRODUCTION.md`](docs/PRODUCTION.md) — deployment and operations runbook
- [`docs/RELEASING.md`](docs/RELEASING.md) — release checklist and repository settings
- [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) — TrumpScript attribution

## License

Crazy Race is available under the [MIT License](LICENSE). TrumpScript is fetched from its MIT-licensed upstream repository during the Docker build; see [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).
