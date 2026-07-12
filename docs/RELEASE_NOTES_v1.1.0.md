# Crazy Race 1.1.0

Crazy Race 1.1.0 is a gameplay-clarity and compatibility release.

## Highlights

- Every new room receives a different deterministic shuffle of the R-generated circuit.
- Terrain tiles now show readable symbols, matching legend colors, and P1/P2 position markers.
- Racer cards show explicit segment progress, current terrain, and an energy meter.
- Unaffordable actions are disabled and also rejected server-side.
- The browser tab now has a Crazy Race SVG icon.
- The real five-codel Piet boost painting is available as a collapsible easter egg.
- The local Docker Compose profile works with Snap Docker's AppArmor restrictions, while the production profile retains stronger `no-new-privileges` hardening.

## Upgrade

```bash
docker compose down --remove-orphans
git pull --ff-only
docker compose up --build
```

Open `http://localhost:8080`.

## Production boundary

The application remains intended for a single-instance hobby, demonstration, or small-community deployment behind HTTPS. Active rooms remain in memory and are cleared when the process restarts.
