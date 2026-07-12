# Production deployment

Crazy Race is production-ready for a **single application instance**. Game rooms live in process memory, so do not run multiple replicas behind a load balancer unless room state is moved to a shared store first.

## Docker package compatibility

Ubuntu's Snap-packaged Docker applies an AppArmor policy that can reject `no-new-privileges` container startup. The default `docker-compose.yml` omits that one flag so local development works with Snap Docker while retaining the non-root user, read-only filesystem, dropped capabilities, and resource limits.

For public production deployment, use Docker Engine from Docker's official packages or Docker Desktop and launch `compose.production.yml`, which retains `no-new-privileges`.

## 1. Configure

Copy the environment template and adjust the limits for the host:

```bash
cp .env.example .env
```

Important settings:

- `HOST_PORT`: loopback port used by the reverse proxy.
- `TRACK_SEED`: deterministic R-generated circuit seed.
- `MAX_CONNECTIONS`: maximum simultaneous accepted TCP connections.
- `MAX_ROOMS`: maximum active in-memory rooms.
- `ROOM_TTL_SECONDS`: inactivity period after which a room is removed.

## 2. Start the hardened container

```bash
docker compose -f compose.production.yml up -d --build
```

The production profile:

- binds only to `127.0.0.1`;
- runs as UID/GID `10001`;
- uses a read-only root filesystem;
- grants no Linux capabilities;
- enables `no-new-privileges`;
- limits PIDs, memory, CPU and log size;
- provides only a small writable `/tmp` tmpfs.

Check readiness:

```bash
docker compose -f compose.production.yml ps
curl --fail http://127.0.0.1:8080/health
```

## 3. Terminate TLS at a reverse proxy

Replace `race.example.com` in `deploy/Caddyfile.example` with the real hostname, install the file in Caddy, and point DNS to the server. The application must not be exposed publicly over plain HTTP because room tokens are bearer credentials contained in game URLs.

The application already emits restrictive CSP, framing, permissions, referrer and cache-control headers. The reverse proxy is responsible for TLS certificates and HSTS.

## 4. Update

```bash
git pull --ff-only
docker compose -f compose.production.yml build --pull
docker compose -f compose.production.yml up -d
```

Verify `/health`, create a test room, join it from a second browser and complete one round after every deployment.

## 5. Back up and scale

There is no durable game data to back up. Restarting the process intentionally removes active rooms. Horizontal scaling is not supported because requests for one room must reach the same in-memory process.

For multi-instance or restart-persistent games, introduce a shared transactional room store before increasing the replica count.
