# Changelog

All notable changes to Crazy Race are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

No unreleased changes.

## [1.1.0] - 2026-07-12

### Added

- Added a proper SVG browser-tab icon.
- Added a collapsible Piet source-painting easter egg to every race page.
- Added visible terrain symbols, color-matched legend cards, and P1/P2 position markers.
- Added explicit segment, current-terrain, and energy meters to racer cards.

### Changed

- Every room now receives its own deterministic shuffle of the R-generated circuit; rematches shuffle again.
- Unaffordable Accelerate and Piet Boost actions are disabled in the UI and rejected by the server.
- Finish progress is capped at the actual segment count while overshoot remains internal to same-round winner resolution.
- The default Compose profile is compatible with Snap Docker; the production profile retains `no-new-privileges`.

### Fixed

- Clarified terrain colors and symbols that were previously too small to read reliably.
- Removed confusing values such as `25 / 20` from the racer display.
- Added CI coverage for favicon delivery, circuit variation, energy validation, terrain legend rendering, and the Piet easter egg.

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

[Unreleased]: https://github.com/Pepitodrop/CrazyRaceGame/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/Pepitodrop/CrazyRaceGame/compare/v1.0.1...v1.1.0
[1.0.1]: https://github.com/Pepitodrop/CrazyRaceGame/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/Pepitodrop/CrazyRaceGame/releases/tag/v1.0.0
