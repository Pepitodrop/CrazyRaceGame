# Crazy Race 1.0.1

Crazy Race 1.0.1 is a production startup compatibility patch.

## Fixed

- Removed Docker Compose's injected `/sbin/docker-init` process from both the local and production profiles.
- Fixed an immediate restart loop on rootless or tightly sandboxed Docker installations where `/sbin/docker-init` could not execute together with the non-root user, dropped capabilities, and `no-new-privileges`.
- Added CI tests that launch and exercise the application through both real Compose profiles, closing the coverage gap that allowed this issue into 1.0.0.

The application process already runs as PID 1, receives Docker's configured `SIGTERM`, and synchronously waits for the short-lived R and TrumpScript child processes it starts, so an additional init process is not required.

## Updating from 1.0.0

```bash
docker compose down
git pull --ff-only
docker compose up --build
```

Then open `http://localhost:8080`.

## Production deployment

```bash
docker compose -f compose.production.yml down
git pull --ff-only
docker compose -f compose.production.yml up -d --build
```

Continue to terminate HTTPS at a reverse proxy as documented in `docs/PRODUCTION.md`.

All gameplay, security controls, and documented single-instance deployment boundaries from 1.0.0 remain unchanged.
