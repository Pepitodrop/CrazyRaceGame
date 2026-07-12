# Crazy Race 1.0.3

Crazy Race 1.0.3 fixes the browser-tab favicon in browsers that enforce the application's Content Security Policy.

## Fixed

- The embedded SVG favicon is now permitted by the `img-src` CSP directive.
- The running-container smoke test now verifies both the favicon markup and the CSP rule required for Firefox to display it.

## Upgrade

```bash
docker compose down --remove-orphans
git pull --ff-only
docker compose build --no-cache
docker compose up
```

Firefox caches favicons aggressively. After upgrading, close and reopen the tab or press `Ctrl+Shift+R` once.
