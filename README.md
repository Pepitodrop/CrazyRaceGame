# Crazy Race Game

[![CI](https://github.com/Pepitodrop/CrazyRaceGame/actions/workflows/ci.yml/badge.svg)](https://github.com/Pepitodrop/CrazyRaceGame/actions/workflows/ci.yml)
[![Release](https://img.shields.io/badge/release-v1.0.1-blue.svg)](CHANGELOG.md)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

Crazy Race 1.0.1 is a Dockerized, browser-based **1v1 turn-based racing game** built around four deliberately unusual languages:

- **Rust**: dependency-free HTTP server, room management, game rules, responsive server-rendered UI, security controls, and the Piet interpreter.
- **R**: deterministic procedural track generation at container startup.
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

Change the deterministic circuit seed with:

```bash
TRACK_SEED=2026 docker compose up --build
```

When updating from 1.0.0, remove the old restart-loop container before starting 1.0.1:

```bash
docker compose down
git pull --ff-only
docker compose up --build
```

## How to play

### Objective

Both racers start at distance `0` with `5` energy. The circuit normally contains `20` numbered segments. The first racer to reach or pass the finish distance wins.

The racer cards show:

- **distance / finish**, for example `9 / 20`;
- the racer's current **energy**;
- a progress bar representing distance along the circuit.

A racer may overshoot the finish, so a final value such as `25 / 20` is valid.

### What happens in one round

1. Both racers choose exactly one action.
2. Each action is evaluated against the terrain the racer occupies at the **start** of that round.
3. Both moves resolve together.
4. The announcer panel reports what happened.
5. If nobody has finished, the next round begins.

The round badge at the top shows the **round you are about to play**. The text under the TrumpScript announcement usually summarizes the **previously completed round**. Therefore, seeing `Round 2` at the top and `Round 1: ...` in the result text is expected.

### Local pass-and-play flow

Local mode is designed for two people sharing one phone, tablet, or computer:

1. Player 1 chooses an action.
2. The page asks you to pass the device to Player 2.
3. Player 1's action remains hidden.
4. Player 2 chooses an action.
5. The round resolves and the result becomes visible.
6. The next round starts with Player 1 choosing first again.

The screenshots that show `Player 1 move` at the start of every new round are therefore correct. The hidden Player 2 handover screen appears between those states.

### Reading the circuit

The numbered boxes are the complete R-generated circuit, not moving car markers. Racer position is shown by the distance values and progress bars above it.

Terrain symbols mean:

- `▰ straight`
- `◒ curve`
- `≈ mud`
- `▲ jump`

Terrain affects movement. Two racers can be on different segments in the same round, so their identical action choices may produce different results.

### Choosing an action

#### Accelerate

- Reliable general-purpose movement.
- Movement is based on the current segment's speed modifier.
- Costs `1` energy.
- Energy never falls below `0`.

Use it when you want predictable progress and do not need to recover energy.

#### Drift

- Strongest on curves.
- Receives a smaller bonus on mud.
- Restores the recovery amount assigned to the current R-generated segment.
- Energy is capped at `10`.

Use it to rebuild energy or exploit a curve.

#### Piet Boost

- Requires at least `3` energy.
- Costs `3` energy when successful.
- Adds the value emitted by the Piet oracle, currently `3`, together with the current segment's speed and boost values.
- Can move up to `9` segments in one round.

If a racer selects Piet Boost with fewer than `3` energy, the boost fails: the racer moves only `1` segment and recovers `1` energy. The result text explicitly says that the racer lacked energy.

### Winning and photo finishes

- If only one racer reaches the finish during a round, that racer wins.
- If both finish in the same round, the racer with the greater final distance wins.
- If both finish on exactly the same distance, Crazy Race applies a deterministic room-and-round tie-breaker.

This explains a result where both racers display `25 / 20` but only one is declared the winner. It is an intentional deterministic photo finish, not a browser-refresh bug or a newly generated random result.

### Example round sequence

Suppose Player 1 begins a round on a curve with enough energy and chooses Drift, while Player 2 chooses Piet Boost without enough energy:

- Player 1 receives the curve bonus and recovers energy.
- Player 2 moves only one segment and regains one energy because the boost failed.
- The next page shows the updated distances and energy values.
- The badge advances to the next round, while the announcer text describes the round that just finished.

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
- all Linux capabilities removed and `no-new-privileges` enabled;
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

`scripts/generate_track.R` produces a tab-separated track with terrain, speed, boost, and energy-recovery attributes. Rust launches it once when the container starts. `data/default_track.tsv` is used only if R generation fails.

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

Crazy Race 1.0.1 is suitable for public release and production deployment as a **single-instance hobby, demo, or small-community service** behind HTTPS. It is not an internet-scale multi-tenant platform.

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
