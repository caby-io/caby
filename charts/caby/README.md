# Caby Helm chart

Deploys the Caby backend (`caby-service`) and frontend (`caby-web`) as two Deployments behind one
Ingress. Backend data lives on a PVC; the backend config is delivered as a mounted Secret.

## Install

Published as an OCI chart on GHCR:

```sh
helm install caby oci://ghcr.io/caby-io/charts/caby \
  --namespace caby --create-namespace \
  --set ingress.web.host=caby.example.com \
  --set-file config.inline=./my-config.yaml
```

From a local checkout, swap the reference for `./charts/caby`.

## Ingress topology

The chart supports single or dual hosts for the frontend/backend:

- `api.host` empty or equal to `web.host` → shared host (web on `/`, API on `/api/v0`). Same-origin,
  no CORS. This is the default.
- `api.host` set to anything else → two hosts. The API owns its host, so it sits at `/v0`; the chart
  wires the backend's CORS allow-origin to the web host.

## Config delivery

The backend reads one `config.yaml` (spaces, users, activation tokens, optional inline OIDC) from
`CABY_CONFIG_PATH`. Provide it either way:

- `config.inline` — the chart renders it into a Secret (default).
- `config.existingSecret` (+ `config.key`) — reference a Secret you manage (sealed-secrets,
  external-secrets, …).

Secret env such as `OIDC_CLIENT_SECRET` goes through `backend.secretEnv` (chart-managed Secret) or
`backend.existingEnvSecret` (`envFrom` a Secret you manage). Env overrides `config.yaml`, so
`urls.backend` / `urls.frontend` in the config file don't need to match the deployment.

## Key values

| Key                         | Default             | Description                                                                                           |
| --------------------------- | ------------------- | ----------------------------------------------------------------------------------------------------- |
| `ingress.web.host`          | `""` (**required**) | Public hostname; drives the web URL and derived env. Install fails if unset while ingress is enabled. |
| `ingress.api.host`          | `""`                | Set to a distinct host for two-host mode; empty = single host.                                        |
| `ingress.tls`               | `true`              | https scheme + TLS block (`tlsSecretName` or `<release>-tls`).                                        |
| `config.inline`             | starter config      | Rendered into a Secret unless `config.existingSecret` is set.                                         |
| `config.existingSecret`     | `""`                | BYO config Secret.                                                                                    |
| `persistence.size`          | `20Gi`              | Backend data PVC size.                                                                                |
| `persistence.existingClaim` | `""`                | Reuse an existing PVC instead of provisioning one.                                                    |
| `backend.secretEnv`         | `{}`                | Map of secret env (e.g. OIDC credentials).                                                            |
| `web.replicas`              | `1`                 | Frontend is stateless and scalable.                                                                   |

The backend runs a single replica with the `Recreate` strategy because it owns a filesystem-backed
store on a `ReadWriteOnce` volume.
