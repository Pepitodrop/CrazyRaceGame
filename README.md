# Crazy Race Game

A Dockerized, browser-based **1v1 turn-based race** built around four deliberately unusual languages:

- **Rust**: HTTP server, room management, game rules, responsive server-rendered UI, and the Piet interpreter.
- **R**: deterministic procedural track generation at container startup.
- **TrumpScript**: the race announcer, executed by the original archived interpreter.
- **Piet**: an image-based boost oracle stored as an ASCII PPM painting.

The browser UI uses only server-rendered HTML and CSS. There is no JavaScript application and no separate frontend service.

## Run it

```bash
docker compose up --build
```

Open `http://localhost:8080` on a computer, phone, or tablet.

Change the deterministic R track seed with:

```bash
TRACK_SEED=2026 docker compose up --build
```

## Playing modes

### Online or LAN room

One player creates a room and shares the six-character code. The second player joins from another browser window or another device that can reach the server.

For another device on the same Wi-Fi network, open the Docker host's local address, for example `http://192.168.1.20:8080`.

### Local pass-and-play

Choose **Local 1v1**, enter two racer names, and play on one computer, phone, or tablet. Player 1 chooses a move, then passes the device to Player 2. The first move is stored without being displayed. After Player 2 chooses, Rust resolves both moves and shows the round result.

## Mobile support

The same server-rendered pages adapt to narrow screens:

- cards and racer panels stack vertically;
- action buttons become large full-width touch targets;
- the circuit can be scrolled horizontally;
- text and spacing scale down on small phones;
- safe-area padding supports modern mobile browsers.

No mobile app installation is required.

## How a race works

Each round, both racers choose one move:

- **Accelerate**: reliable speed, costs one energy.
- **Drift**: strongest on curves and restores energy according to the R-generated segment.
- **Piet Boost**: consumes three energy and adds the number emitted by the Piet painting.

In online mode, the round resolves after both players submit. In local mode, the same process happens sequentially on one device. The first racer across the final segment wins. If both cross in the same round, overshoot distance decides; an exact tie uses a deterministic room tie-breaker.

## Language architecture

### Rust

The dependency-free Rust application uses only the standard library. It owns all mutable game state in memory, renders every page on the server, validates room and local-game tokens, resolves turns, and interprets the Piet oracle.

The source is split into `src/part1.rs` through `src/part5.rs` and assembled by `src/main.rs` with `include!` so the resulting program remains one Rust binary.

### R

`scripts/generate_track.R` produces a tab-separated track with terrain, speed, boost, and energy-recovery attributes. Rust launches it once when the container starts. `data/default_track.tsv` is used only if R generation fails.

### TrumpScript and the limited Python exception

The Docker build clones the original archived repository at the pinned commit:

```text
samshadwell/TrumpScript@3793b905925b55c0296b066586c7d612c7220ca0
```

The announcer source is `announcer/race.tr`, and Rust executes it through the original `TRUMP --shut-up` entrypoint. Python is installed **only** because the upstream TrumpScript interpreter is implemented in Python. No game logic, networking, rendering, track generation, or Piet execution is written in Python.

A compatibility-only AST shim maps the interpreter's removed legacy `Module`, `Num`, `Str`, and `NameConstant` constructors to their modern Python equivalents. The TrumpScript grammar and runtime behavior remain upstream code.

### Piet

`piet/boost_oracle.ppm` is executable source code as an image. Its codel transitions perform:

1. `push 3`
2. `out(number)`

Rust reads the PPM, interprets those Piet transitions, and uses the emitted `3` as the boost modifier.

## Validation

```bash
cargo test --locked
docker build -t crazy-race-game .
docker run --rm --entrypoint /opt/trumpscript/bin/TRUMP \
  crazy-race-game --shut-up /app/announcer/race.tr
```

The GitHub Actions workflow runs the Rust tests, release build, Docker build, original TrumpScript interpreter, and container health check.

## Operational limits

This is a compact game prototype. Rooms and local games are stored in memory and disappear when the container restarts. Player tokens appear in the browser URL, so deploy it behind HTTPS before exposing it beyond a trusted network.

## License and attribution

This repository is MIT licensed. TrumpScript is fetched from its own MIT-licensed upstream repository during the Docker build; see `THIRD_PARTY_NOTICES.md`.
