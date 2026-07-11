# Releasing Crazy Race

Crazy Race follows Semantic Versioning. Release tags use the form `vMAJOR.MINOR.PATCH` and must point to a fully validated commit on `main`.

## Release checklist

1. Confirm that all intended changes are merged and no pull requests remain open.
2. Confirm that `Cargo.toml` contains the release version.
3. Move completed entries from `Unreleased` into a dated section in `CHANGELOG.md`.
4. Run the release checks:

   ```bash
   rustfmt --edition 2021 src/*.rs
   cargo clippy --all-targets --locked -- -D warnings
   cargo test --locked
   cargo build --release --locked
   docker compose -f compose.production.yml config --quiet
   docker build -t crazy-race-game:release .
   ```

5. Start the hardened container and run the complete smoke test:

   ```bash
   docker run -d --name crazy-race-release \
     -p 127.0.0.1:18080:8080 \
     --read-only \
     --tmpfs /tmp:rw,noexec,nosuid,nodev,size=16m \
     --cap-drop ALL \
     --security-opt no-new-privileges \
     --pids-limit 256 \
     --memory 256m \
     --cpus 1 \
     crazy-race-game:release

   bash scripts/smoke-test.sh http://127.0.0.1:18080
   docker rm -f crazy-race-release
   ```

6. Verify the repository contains no local configuration, credentials, private keys, personal data, or private infrastructure details.
7. Create an annotated tag from the validated `main` commit:

   ```bash
   git switch main
   git pull --ff-only
   git tag -a v1.0.0 -m "Crazy Race 1.0.0"
   git push origin v1.0.0
   ```

8. Create a GitHub Release from that tag, title it `Crazy Race 1.0.0`, paste the prepared release notes, and leave both **pre-release** and **draft** disabled.
9. Verify the release page, source archives, CI badge, license display, and documentation links while signed out of GitHub.

## Public repository settings

Before publishing the first release:

- set repository visibility to **Public**;
- add a concise repository description and topics such as `rust`, `r`, `docker`, `game`, `trumpscript`, and `piet`;
- enable private vulnerability reporting;
- enable Dependabot alerts and security updates;
- protect `main` by requiring the `CI / test` check before merging;
- enable automatic deletion of merged branches;
- keep force pushes and branch deletion disabled for `main`.

## Rollback

Do not move or overwrite a published version tag. If a release is defective, document the problem, publish a corrective patch release, and mark the affected release notes clearly. Container restarts remove active rooms because runtime state is intentionally in memory.
