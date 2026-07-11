# Security Policy

## Supported versions

| Version | Supported |
| --- | --- |
| 1.0.x | Yes |
| Earlier or unreleased snapshots | No |

Security fixes are applied to the current `main` branch and the latest supported release.

## Reporting a vulnerability

Report suspected vulnerabilities privately through GitHub's **Report a vulnerability** flow:

`https://github.com/Pepitodrop/CrazyRaceGame/security/advisories/new`

Do not include exploitable details in a public issue, pull request, discussion, or log excerpt.

Include the affected version, route or component, reproduction steps, expected impact, and any suggested mitigation. Remove unrelated personal data, production host details, and access tokens. Reports will be reviewed before public disclosure.

## Deployment boundary

Crazy Race is designed as a single-instance service behind a TLS-terminating reverse proxy. Access tokens are bearer credentials contained in game URLs and must be protected with HTTPS. Active rooms are held in memory, disappear on restart, and are intentionally not shared between replicas.

Security reports about deployments that remove these documented controls may still be useful, but the report should identify the changed boundary clearly.
