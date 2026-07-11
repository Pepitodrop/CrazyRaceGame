# Crazy Race Game

Crazy Race v1.0 is a Dockerized, browser-based **1v1 turn-based racing game** built around four deliberately unusual languages:

- **Rust**: dependency-free HTTP server, room management, game rules, responsive server-rendered UI, security controls, and the Piet interpreter.
- **R**: deterministic procedural track generation at container startup.
- **TrumpScript**: race announcements, executed by the original archived interpreter.
- **Piet**: an image-based boost oracle stored as an ASCII PPM painting.

The browser UI uses only server-rendered HTML and CSS. There is no JavaScript application and no separate frontend service.

## Run locally or on a LAN

```bash
docker compose up --build
```

Open `http://localhost:8080` on a computer, phone, or tablet. Another device on the same Wi-Fi network can open the Docker host's local address, for example `http://192.168.1.20:8080`.

Change the deterministic circuit seed with:

```bash
TRACK_SEED=2026 docker compose up --build
```

## Production deployment

The hardened production profile binds the application to loopback so it can sit behind a TLS reverse proxy:

```bash
cp .env.example .env
docker compose -f compose.production.yml up -d --build
```

Use `deploy/Caddyfile.example` as a TLS reverse-proxy starting point. Full deployment, update, scaling and operational instructions are in [`docs/PRODUCTION.md`](docs/PRODUCTION.md).

Production safeguards include:

- a non-root UID/GID;
- a read-only root filesystem and small writable `/tmp` tmpfs;
- all Linux capabilities removed and `no-new-privileges` enabled;
- CPU, memory, process and log limits;
- cryptographically random 256-bit bearer tokens on Linux;
- request-header, request-body, URI, connection and room limits;
- automatic expiry of inactive rooms;
- CSP, cache-control, referrer, framing, permissions and cross-origin response headers;
- an HTTP-aware readiness check;
- end-to-end CI tests against the hardened container configuration.

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

## Race rules

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
docker build -t crazy-race-game .
bash scripts/smoke-test.sh http://127.0.0.1:8080
```

GitHub Actions additionally validates the production Compose file, verifies the non-root image user, executes the original TrumpScript interpreter, and runs the complete online and local gameplay smoke test inside a read-only, capability-free container.

## Operational boundary

Crazy Race is production-ready as a **single-instance service**. Active rooms are stored in memory and disappear when the process restarts. Do not deploy multiple replicas without first moving room state to a shared transactional store.

Player tokens are bearer credentials contained in game URLs. Public deployments must use HTTPS and the application sends `Referrer-Policy: no-referrer` plus `Cache-Control: no-store` to reduce leakage and caching risk.

## Security, license and attribution

See [`SECURITY.md`](SECURITY.md) for private vulnerability reporting guidance. This repository is MIT licensed. TrumpScript is fetched from its MIT-licensed upstream repository during the Docker build; see `THIRD_PARTY_NOTICES.md`.
