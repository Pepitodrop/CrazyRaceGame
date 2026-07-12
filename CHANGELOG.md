# Changelog

All notable changes to Crazy Race are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

No unreleased changes.

## [1.0.2] - 2026-07-13

### Added

- Browser tab favicon.
- Visible terrain symbols inside each circuit segment and a larger color-coded legend.
- Track fingerprint for identifying the generated circuit.
- Magnified Piet image-program easter egg at the bottom of every page.
- Separate energy meter, current-segment details, terrain status, and explicit action formulas.

### Changed

- The default track seed is now `0`, which generates a fresh circuit on each container start. Positive seeds remain reproducible.
- The race counter is clamped to the finish distance while an overshoot finish score is shown separately.
- Piet Boost is disabled in the normal UI when the racer has fewer than three energy.
- The default local Compose profile omits `no-new-privileges` for Snap Docker compatibility while keeping the non-root user, read-only filesystem, dropped capabilities, and resource limits. The production profile remains hardened for standard Docker Engine.

### Fixed

- Clarified track colors, symbols, energy changes, and segment progression directly in the game UI.
- Prevented confusing values such as `25 / 20` in the primary segment counter.
- Replaced the fixed default track with a fresh generated track after every container restart.

## [1.0.1] - 2026-07-12

### Fixed

- Removed Docker Compose's injected `/sbin/docker-init` process from local and production profiles. On some rootless or tightly sandboxed Docker installations it could not execute together with the non-root user, dropped capabilities, and `no-new-privileges`, causing an immediate restart loop.
- Added CI coverage that launches the application through the actual default and production Compose profiles, rather than validating only an equivalent `docker run` command.

## [1.0.0] - 2026-07-12

### Added

- Browser-based 1v1 turn-based racing with online room codes and same-device pass-and-play.
- Responsive server-rendered interface for desktop, tablet, and mobile browsers.
- Dependency-free Rust HTTP server, game engine, room management, and Piet interpreter.
- Deterministic R-generated tracks with a static fallback circuit.
- Original pinned TrumpScript interpreter for race announcements.
- Executable Piet PPM program that supplies the boost modifier.
- Docker and Docker Compose deployment for local, LAN, and reverse-proxied production use.
- Production runbook, Caddy TLS example, health check, and configurable resource limits.

### Security

- 256-bit bearer tokens sourced from the operating system random generator on Linux.
- Hidden online and local moves until both players have submitted.
- Request-header, request-body, URI, connection, room-count, and room-lifetime limits.
- Automatic expiration of inactive in-memory rooms.
- Restrictive CSP, cache-control, referrer, framing, permissions, and cross-origin headers.
- Non-root container execution with a read-only filesystem, dropped capabilities, and `no-new-privileges`.

### Known limitations

- Active rooms are stored only in memory and disappear when the process restarts.
- Horizontal scaling is not supported until room state is moved to a shared transactional store.
- Public deployments must terminate HTTPS at a reverse proxy because access tokens are bearer credentials contained in game URLs.

[Unreleased]: https://github.com/Pepitodrop/CrazyRaceGame/compare/v1.0.2...HEAD
[1.0.2]: https://github.com/Pepitodrop/CrazyRaceGame/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/Pepitodrop/CrazyRaceGame/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/Pepitodrop/CrazyRaceGame/releases/tag/v1.0.0
