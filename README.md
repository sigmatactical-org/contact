# sigma-contact

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
| `CONTACT_DATA_PATH` | JSON database path (default `data/contacts.json`) |
| `CONTACT_IDENTITY_ISSUER_URL` | OIDC issuer / realm URL (e.g. `http://127.0.0.1:8101/realms/multcorp`) |
| `CONTACT_IDENTITY_CLIENT_ID` | Service-account client id for Admin API |
| `CONTACT_IDENTITY_CLIENT_SECRET` | Service-account client secret |

Identity sync requires a Keycloak client with **service accounts enabled** and the **view-users** role on **realm-management**. In the dev realm, you can reuse the `identity` client credentials and assign that role to `service-account-identity`.

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
./scripts/prepare-local.sh
cargo run -p sigma-contact
```

Under the sigma workspace (`sigma/commerce/contact`):

```bash
cd sigma/commerce/contact && ./scripts/prepare-local.sh && cargo run -p sigma-contact
# or: (cd sigma/commerce && ./scripts/prepare-local.sh && cargo run -p sigma-contact)
```

Open http://localhost:8080

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

Mount a volume at `/app/data` (or set `CONTACT_DATA_PATH`) so contact data persists across restarts.

## License

MIT OR Apache-2.0
