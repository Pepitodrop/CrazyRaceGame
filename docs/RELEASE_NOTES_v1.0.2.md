# Crazy Race 1.0.2

Crazy Race 1.0.2 is a gameplay-clarity and Docker-compatibility patch.

## Highlights

- Added a browser-tab favicon.
- Added visible terrain symbols to every track segment and a larger color-coded legend.
- Added a Track ID so generated circuits can be distinguished.
- Added a magnified Piet image-program easter egg at the bottom of every page.
- Added a separate energy meter, current-segment status, terrain label, and explicit action formulas.
- Disabled Piet Boost in the UI when the racer has insufficient energy.
- Clamped the visible race counter to the finish line while preserving the overshoot finish score.
- Changed the default seed to generate a fresh track on each container start; positive seeds remain reproducible.
- Made the default local Compose profile compatible with Snap Docker's AppArmor restrictions.

## Docker

Local development:

```bash
docker compose down --remove-orphans
git pull --ff-only
docker compose up --build
```

Production deployments should continue to use standard Docker Engine and the hardened profile:

```bash
docker compose -f compose.production.yml up -d --build
```

## Scope

The game remains intended for one application instance behind HTTPS. Rooms are stored in memory and are cleared when the process restarts.
