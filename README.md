# Crazy Race Game

A Dockerized, browser-based **1v1 turn-based race** built around four deliberately unusual languages:

- **Rust**: HTTP server, room management, game rules, server-rendered UI, and the Piet interpreter.
- **R**: deterministic procedural track generation at container startup.
- **TrumpScript**: the race announcer, executed by the original archived interpreter.
- **Piet**: an image-based boost oracle stored as an ASCII PPM painting.

The browser UI uses only server-rendered HTML and CSS. There is no JavaScript application and no separate frontend service.

## Run it

```bash
docker compose up --build
```

Open `http://localhost:8080` in two browser windows. One player creates a room and shares the six-character code; the other joins it.

Change the deterministic R track seed with:

```bash
TRACK_SEED=2026 docker compose up --build
```

## How a race works

Each round, both racers secretly lock in one move:

- **Accelerate**: reliable speed, costs one energy.
- **Drift**: strongest on curves and restores energy according to the R-generated segment.
- **Piet Boost**: consumes three energy and adds the number emitted by the Piet painting.

The round resolves only after both players submit. The first racer across the final segment wins. If both cross in the same round, overshoot distance decides; an exact tie uses a deterministic room tie-breaker.

## Language architecture

### Rust

`src/main.rs` contains a dependency-free HTTP server using the Rust standard library. It owns all mutable game state in memory, renders every page on the server, validates room tokens, resolves simultaneous turns, and interprets the Piet oracle.

### R

`scripts/generate_track.R` produces a tab-separated track with terrain, speed, boost, and energy-recovery attributes. Rust launches it once when the container starts. `data/default_track.tsv` is used only if R generation fails.

### TrumpScript and the limited Python exception

The Docker build clones the original archived repository at the pinned commit:

```text
samshadwell/TrumpScript@3793b905925b55c0296b066586c7d612c7220ca0
```

The announcer source is `announcer/race.tr`, and Rust executes it through the original `TRUMP --shut-up` entrypoint. Python is installed **only** because the upstream TrumpScript interpreter is implemented in Python. No game logic, networking, rendering, track generation, or Piet execution is written in Python.

A one-line compatibility patch adds the `type_ignores` field required by modern Python's AST API. The TrumpScript language implementation itself remains upstream code.

### Piet

`piet/boost_oracle.ppm` is executable source code as an image. Its codel transitions perform:

1. `push 3`
2. `out(number)`

Rust reads the PPM, interprets those Piet transitions, and uses the emitted `3` as the boost modifier.

## Project layout

```text
.
├── announcer/race.tr
├── data/default_track.tsv
├── piet/boost_oracle.ppm
├── scripts/generate_track.R
├── src/main.rs
├── Dockerfile
└── docker-compose.yml
```

## Validation

```bash
cargo test --locked
docker build -t crazy-race-game .
docker run --rm --entrypoint /opt/trumpscript/bin/TRUMP \
  crazy-race-game --shut-up /app/announcer/race.tr
```

The GitHub Actions workflow runs the Rust tests, release build, Docker build, original TrumpScript interpreter, and container health check.

## Operational limits

This is a compact game prototype. Rooms are stored in memory and disappear when the container restarts. Player tokens appear in the browser URL, so deploy it behind HTTPS before exposing it beyond a trusted network.

## License and attribution

This repository is MIT licensed. TrumpScript is fetched from its own MIT-licensed upstream repository during the Docker build; see `THIRD_PARTY_NOTICES.md`.
