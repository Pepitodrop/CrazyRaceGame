## Summary

- Explain what changed and why.

## Validation

- [ ] `cargo clippy --all-targets --locked -- -D warnings`
- [ ] `cargo test --locked`
- [ ] `docker compose up -d --build`
- [ ] `bash scripts/smoke-test.sh http://127.0.0.1:8080`

## Release impact

- [ ] Version and changelog updated when user-visible behavior changed.
- [ ] Documentation updated.
