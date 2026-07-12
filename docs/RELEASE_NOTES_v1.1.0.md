# Crazy Race 1.1.0

Crazy Race 1.1.0 makes Piet an active, changing part of every race.

## Adaptive Piet oracle

The image program now reads four deterministic numeric inputs:

- current round;
- terrain code;
- comeback gap bucket;
- energy bucket.

It calculates `1 + ((round + terrain + comeback + energy) mod 4)` and outputs a boost between `+1` and `+4`. The UI previews the value before a player commits the move, while Rust validates and clamps every result.

## Interpreter improvements

The bounded linear Piet runtime now supports numeric input and comparison in addition to the existing stack arithmetic, modulo, duplicate, push, and output operations.

## Compatibility and deployment

The Docker, standard Compose, Snap-compatible local Compose, and hardened production Compose behavior are unchanged from v1.0.3. Rooms remain in memory and public deployments still require HTTPS.
