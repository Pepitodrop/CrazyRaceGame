## Summary

Describe the problem and the implemented change.

## Validation

- [ ] `rustfmt --edition 2021 src/*.rs`
- [ ] `cargo clippy --all-targets --locked -- -D warnings`
- [ ] `cargo test --locked`
- [ ] `cargo build --release --locked`
- [ ] Docker image builds successfully
- [ ] Relevant online or local gameplay path was tested

## Safety and compatibility

- [ ] No credentials, `.env` files, tokens, private keys, personal data, or production host details are included
- [ ] Tests cover changed behavior or the reason they are unnecessary is explained
- [ ] Documentation and `CHANGELOG.md` are updated when user-visible behavior changes
- [ ] Mobile layout and accessibility were considered
- [ ] Python remains limited to the upstream TrumpScript compatibility path
- [ ] The single-instance in-memory deployment boundary is preserved or the architectural migration is documented

## Additional notes

Include screenshots, logs with sensitive values removed, migration notes, or follow-up work where relevant.
