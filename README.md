# sigma-contact

[![CI](https://github.com/sigmatactical-org/contact/actions/workflows/ci.yml/badge.svg)](https://github.com/sigmatactical-org/contact/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![MSRV](https://img.shields.io/badge/MSRV-1.97.0-blue.svg)](https://www.rust-lang.org)

Contact directory for Sigma Tactical Group. Pulls users from the identity provider (Keycloak), stores external contacts locally, and exposes a simple web UI plus JSON API.

**Internal / admin tool** — not customer-facing. The public storefront is [sigma-store](https://github.com/sigmatactical-org/store); this service is reached only through the [sigma-identity](https://github.com/sigmatactical-org/identity) authenticated proxy.

Repository: https://github.com/sigmatactical-org/contact

Shared site chrome comes from [sigma-theme](https://github.com/sigmatactical-org/sigma-theme).

## Features

- **Identity sync** — import enabled realm users via Keycloak Admin API (client credentials)
- **External contacts** — add, edit, and delete contacts outside the identity directory
- **Web UI** — server-rendered pages for browsing and managing contacts
- **JSON API** — programmatic CRUD and sync for integration behind [sigma-identity](https://github.com/sigmatactical-org/identity)

## Configuration

| Variable | Purpose |
|----------|---------|
| `PORT` | Listen port (default `8080`) |
| `DATABASE_URL` | PostgreSQL connection URL (default `postgres://sigma:sigma@127.0.0.1:5432/sigma`) |
| `CONTACT_IDENTITY_ISSUER_URL` | OIDC issuer / realm URL (e.g. `http://127.0.0.1:8101/realms/multcorp`) |
| `CONTACT_IDENTITY_CLIENT_ID` | Service-account client id for Admin API |
| `CONTACT_IDENTITY_CLIENT_SECRET` | Service-account client secret |
| `HUMAN_CHECK_HMAC_SECRET` | Bot-check HMAC secret for the public form (≥32 characters) |
| `HUMAN_CHECK_KEY_SECRET` | Bot-check key secret for the public form (≥32 characters) |
| `HUMAN_CHECK_DISABLED` | Set `true` to accept unverified submissions (local dev only) |

The public contact form is guarded by [sigma-human-check](https://github.com/sigmatactical-org/human-check), which fails closed: the service refuses to start unless both secrets are set or `HUMAN_CHECK_DISABLED=true`.

Identity sync requires a Keycloak client with **service accounts enabled** and the **view-users** role on **realm-management**. In the dev realm, run `platform/scripts/seed-keycloak-dev-users.sh` to grant `view-users` on `service-account-identity`.

## API

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/contacts` | List all contacts |
| `GET` | `/contacts/{id}` | Get one contact |
| `POST` | `/contacts` | Create external contact (JSON) |
| `PUT` | `/contacts/{id}` | Update external contact |
| `DELETE` | `/contacts/{id}` | Delete external contact |
| `POST` | `/contacts/sync` | Pull users from identity |

Identity-sourced contacts are read-only via the API; re-sync to refresh them.

### Behind sigma-identity

Point identity at this service, for example:

```bash
IDENTITY_PROXY_TARGET=http://127.0.0.1:8080/
```

Browser clients call `/api/contacts` on the identity host (with session + CSRF); identity forwards the request with a Bearer token attached.

## Development

Standalone clone:

```bash
HUMAN_CHECK_DISABLED=true cargo run -p sigma-contact
```

Under the sigma workspace (`sigma/it/contact`):

```bash
cd sigma/it/contact
HUMAN_CHECK_DISABLED=true cargo run -p sigma-contact
```

Drop `HUMAN_CHECK_DISABLED` and export the two `HUMAN_CHECK_*` secrets instead to exercise the real proof-of-work widget; `HUMAN_CHECK_COST=1500` keeps it instant.

Open http://localhost:8080

### Shared crates

`sigma-theme`, `sigma-pg`, and `sigma-human-check` are pinned git
dependencies, so a fresh clone builds with nothing but `cargo`: the revision
in `Cargo.toml` is fetched, and `build.rs` writes the `askama.toml` that points
at sigma-theme's templates wherever Cargo put them.

When one of those crates is checked out beside this repo and you are editing it,
link the checkouts so your edits are picked up without a push:

```bash
./scripts/prepare-local.sh
```

That writes `[patch]` entries into `.cargo/config.toml` (gitignored) for the
crates it finds and leaves the rest on their pinned revision; it prints what it
linked. Undo by deleting the file. Note that building against a linked checkout
rewrites `Cargo.lock` into path form — don't commit that; `platform`'s
`scripts/relock.sh` restores the git-resolved lockfile CI expects.

Bumping a shared crate is `platform/scripts/pin-shared-revs.sh <crate>` after
that crate is pushed, which updates every consumer's pin at once.

Example local identity sync (with dev-stack Keycloak running):

```bash
export CONTACT_IDENTITY_ISSUER_URL=http://127.0.0.1:8101/realms/multcorp
export CONTACT_IDENTITY_CLIENT_ID=identity
export CONTACT_IDENTITY_CLIENT_SECRET=8d476311-2577-4104-b9e4-7dc2cc381be8
cargo run -p sigma-contact
```

Then use **Sync from identity** in the web UI or `POST /contacts/sync`.

## Docker

Release is in **`.github/workflows/release.yml`** when configured. Locally:

```bash
./scripts/docker-build.sh
docker build -f Dockerfile build/image
```

Data is stored in the shared PostgreSQL `contact` schema (`contact.contacts` JSONB table). Postgres runs in the [platform](https://github.com/sigmatactical-org/platform) kind stack — port-forward for local `cargo run`:

```bash
cd platform && ./scripts/postgres-dev.sh port-forward-bg && ./scripts/postgres-dev.sh migrate
```

## Brand & artwork

© Sigma Tactical Group. **All rights reserved.**

The Sigma Tactical Group name, logos, marks, artwork, and visual identity are **proprietary**. They are not covered by this repository's source-code license. See [BRANDING.md](BRANDING.md).

## License

MIT OR Apache-2.0 for **source code** only. Branding remains proprietary.
