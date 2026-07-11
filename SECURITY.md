# Security Policy

## Supported version

Security fixes are applied to the current `main` branch and the latest published release.

## Reporting a vulnerability

Please report suspected vulnerabilities privately through GitHub's **Report a vulnerability** security-advisory flow when it is available for this repository. Do not include exploitable details in a public issue.

Include the affected route or component, reproduction steps, expected impact, and any suggested mitigation. Reports will be reviewed before public disclosure.

## Deployment boundary

Crazy Race is designed as a single-instance service behind a TLS-terminating reverse proxy. Access tokens are bearer credentials and must be protected with HTTPS. Active rooms are held in memory and are intentionally not shared between replicas.
