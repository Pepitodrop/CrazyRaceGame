# Changelog

All notable changes to Crazy Race are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

No unreleased changes.

## [1.0.0] - 2026-07-11

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

[Unreleased]: https://github.com/Pepitodrop/CrazyRaceGame/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/Pepitodrop/CrazyRaceGame/releases/tag/v1.0.0
