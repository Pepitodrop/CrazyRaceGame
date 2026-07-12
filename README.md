# Crazy Race Game

[![CI](https://github.com/Pepitodrop/CrazyRaceGame/actions/workflows/ci.yml/badge.svg)](https://github.com/Pepitodrop/CrazyRaceGame/actions/workflows/ci.yml)
[![Release](https://img.shields.io/badge/release-v1.0.3-blue.svg)](CHANGELOG.md)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

Crazy Race 1.0.3 is a Dockerized, browser-based **1v1 turn-based racing game** powered by Rust, R, TrumpScript, and Piet.

- **Rust** runs the HTTP server, rooms, game rules, HTML UI, security controls, and Piet interpreter.
- **R** generates the 20-segment circuit when the container starts.
- **TrumpScript** produces the race-announcer lines through its original Python interpreter.
- **Piet** is an image program whose output supplies the boost value.

The frontend is server-rendered HTML and CSS. There is no JavaScript application.

## Quick start

```bash
git clone https://github.com/Pepitodrop/CrazyRaceGame.git
cd CrazyRaceGame
docker compose up --build
```

Open `http://localhost:8080`.

The default Compose profile is compatible with Docker Desktop, standard Docker Engine, and Snap Docker. The hardened production profile is intended for standard Docker Engine behind HTTPS.

## How to play

### Goal

Both racers start at distance `0` with `5 / 10` energy. Reach segment `20` first.

The racer cards now show:

- the clamped race counter, such as `9 / 20`;
- the current segment label;
- energy as a number and a separate energy bar;
- whether Piet Boost is currently available;
- the race progress bar.

If a racer overshoots the finish, the normal counter stops at `20 / 20` and the separate finish score records the overshoot used for photo finishes.

### One round

1. Each racer chooses one action.
2. The action is evaluated against the terrain occupied at the start of the round.
3. Both moves resolve together.
4. Distance and energy are updated.
5. The announcer explains the completed round.

The badge at the top shows the round you are about to play. The announcer text describes the previous completed round.

### Actions and energy

#### Accelerate

- Movement: base `2` plus the current terrain speed modifier.
- Energy: costs `1`.
- Best for reliable progress.

#### Drift

- Receives its largest movement bonus on curves and a smaller bonus on mud.
- Restores the current segment's R-generated recovery value.
- Energy is capped at `10`.

#### Piet Boost

- Requires at least `3` energy.
- Costs `3` energy.
- Adds the Piet oracle output, currently `3`, plus the terrain speed and boost values.
- The button is disabled when the current racer does not have enough energy.

### Local pass-and-play

1. Player 1 chooses.
2. Pass the device to Player 2.
3. Player 1's move stays hidden.
4. Player 2 chooses.
5. The round resolves.

### Reading the circuit

Every track box now contains both a terrain symbol and its segment number:

| Color | Symbol | Terrain | Effect |
|---|---:|---|---|
| Blue-gray | `▰` | Straight | Usually favors acceleration |
| Purple | `◒` | Curve | Best terrain for Drift |
| Brown | `≈` | Mud | Usually slows movement |
| Teal | `▲` | Jump | Can add a boost bonus |

The game also displays a Track ID so it is easy to see when a generated circuit changes.

By default, `TRACK_SEED=0` generates a fresh circuit each time the container starts. The track remains fixed during that running server session so every racer in a room uses the same circuit. To reproduce a circuit, provide a positive seed:

```bash
TRACK_SEED=2026 docker compose up --build
```

### Winning

- One finisher: that racer wins.
- Both finish in one round: the larger finish score wins.
- Exact equal finish scores: a deterministic room-and-round photo-finish rule selects the winner.

## Piet easter egg

At the bottom of every page, expand **Piet easter egg** to see a magnified rendering of the actual `piet/boost_oracle.ppm` image program used by the game.

## Mobile support

The same pages adapt to narrow screens: cards stack, controls become full-width touch targets, the circuit scrolls horizontally, and safe-area padding is applied.

## Production deployment

```bash
cp .env.example .env
docker compose -f compose.production.yml up -d --build
```

The production profile binds to loopback for use behind a TLS reverse proxy. See [`docs/PRODUCTION.md`](docs/PRODUCTION.md) and `deploy/Caddyfile.example`.

Production controls include:

- non-root UID/GID;
- read-only root filesystem and limited writable `/tmp`;
- dropped Linux capabilities;
- `no-new-privileges` in the production profile;
- CPU, memory, process, connection, request, room-count, and room-lifetime limits;
- 256-bit bearer tokens;
- restrictive security and cache headers;
- HTTP-aware health checks;
- CI tests for Rust, Docker, and both Compose profiles.

### Snap Docker note

Canonical's Snap Docker AppArmor profile rejects process execution when `no-new-privileges` is used. Therefore the default local Compose profile omits that one option while retaining the non-root user, read-only filesystem, dropped capabilities, and resource limits. The production profile retains `no-new-privileges` for standard Docker Engine.

## Architecture

### Rust

The Rust standard-library application owns HTTP, state, rendering, movement rules, validation, rate/resource limits, and Piet execution.

### R

`scripts/generate_track.R` creates terrain, speed, boost, and energy-recovery values. A seed of `0` selects a fresh time-based seed; positive values are reproducible. `data/default_track.tsv` is used only if R generation fails.

### TrumpScript

The Docker build fetches `samshadwell/TrumpScript@3793b905925b55c0296b066586c7d612c7220ca0`. Python is installed only for that upstream interpreter.

### Piet

`piet/boost_oracle.ppm` is executable image source. Its codel transitions push and output `3`, which Rust applies as the oracle boost modifier.

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

## Support boundary

Crazy Race 1.0.3 is suitable for public release as a **single-instance hobby, demo, or small-community service** behind HTTPS. Rooms are held in memory and disappear after restart. Horizontal replicas require a shared transactional room store first.

Player tokens are bearer credentials in game URLs. Public deployments must use HTTPS.

## Documentation

- [`CHANGELOG.md`](CHANGELOG.md)
- [`CONTRIBUTING.md`](CONTRIBUTING.md)
- [`SECURITY.md`](SECURITY.md)
- [`docs/PRODUCTION.md`](docs/PRODUCTION.md)
- [`docs/RELEASING.md`](docs/RELEASING.md)
- [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)

## License

Crazy Race is available under the [MIT License](LICENSE). See [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) for TrumpScript attribution.
