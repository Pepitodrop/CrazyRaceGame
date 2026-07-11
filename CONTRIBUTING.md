# Contributing to Crazy Race

Thank you for improving Crazy Race. Small, focused pull requests are easiest to review.

## Development setup

Install Docker with Docker Compose and Rust through `rustup`. The repository pins Rust 1.82.0, including `rustfmt` and Clippy, through `rust-toolchain.toml`.

Run the complete application:

```bash
docker compose up --build
```

Run the Rust checks locally:

```bash
rustfmt --edition 2021 src/*.rs
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --release --locked
```

For the full browser-level smoke test, start the application and run:

```bash
bash scripts/smoke-test.sh http://127.0.0.1:8080
```

## Pull requests

1. Create a branch from the latest `main`.
2. Keep the change scoped to one fix or feature.
3. Add or update tests for behavior changes.
4. Update the README, changelog, or production documentation when relevant.
5. Confirm that no credentials, `.env` files, private keys, personal data, or production host details are committed.
6. Open a pull request and complete the checklist in the template.

The project intentionally avoids a JavaScript application and external Rust dependencies. Proposals that change those architectural constraints should explain the trade-off clearly.

## Commit and code style

- Use clear imperative commit messages.
- Keep Rust free of Clippy warnings rather than suppressing them without justification.
- Preserve server-side HTML rendering and mobile usability.
- Keep Python limited to the upstream TrumpScript interpreter compatibility path.
- Use LF line endings; `.gitattributes` enforces this for repository text files.

## Reporting security issues

Do not open public issues for vulnerabilities. Follow the private process in [SECURITY.md](SECURITY.md).

## License

By contributing, you agree that your contribution is licensed under the repository's MIT License.
