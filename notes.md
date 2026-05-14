# Files response

```json
{
  "name": "Name of the entry",
  "path": "relative/path/of/entry",
  ...
}
```

# Dir structure

```
/app/cabynet
  ├── /users
  ├── /spaces
    ├── /user
      ├── /live
      ├── /meta
      ├── /shares
      └── /uploads
    └── /media
      ├── /live
      ├── /meta
      ├── /shares
      └── /uploads
  └── config.yaml
```

## User dir contents

```
/app/cabynet/users
  ├── /first_user
    ├── password      # user's salted & hashed p/w
    ├── profile.yaml  # user's profile, config, & preferences info
    ├── session_...   # an active user session
    └── ...more
```

# Request Paths

/list/{space}/

# Configs

```yaml
# config.yaml
---
# registration_enabled: false

# paths:
#   users_path: "users"
#   spaces_path: "spaces"
web:
  domain: "" # todo for CORS

auth:
  passwords:
    enabled: true
  # oidc using discovery
  oidc:
    issuer_url: "auth.oidc.com"
  # oidc manual
  oidc:
    issuer_url: "auth.oidc.com",
    authorization_endpoint: "",
    token_endpoint: "",
    jwks_uri: "",
    userinfo_endpoint: "" # optional

spaces:
  - name: home
    archetype: users
    path: /some/other/path # optional override
    readonly: false
  - name: media

roles:
  - name: admin
    global_permissions: "*"
  - name: family
    spaces:
      - name: home
        permissions: # todo
          - "rw:/"

users:
  - name: caby_guy
    email: caby_guy@caby.io
    activation_token: OHQFhErYIM7xK8gMtf9emXt4LssVp5ibBs3MgJXTBQXbw8Cs4HUyWv1HdXjJyUL5
    spaces:
      - name: home
        permissions: "*"
```

# Upload Management

```mermaid
graph TD;
    A["user submits files for upload"];
    B["create upload group"];
    C["submit upload group to upload manager queue"];
    D["(per-file) start uploading"];

    E_1["hash file"];
    E_2["submit hash to backend"];

    F_1["(per-chunk) upload chunks"];

    G["stage file"];
    H["publish group"];

    A-->B-->C-->D;
    D-->E_1-->E_2-->G;
    D-->F_1-->G-->H;
```

- Should we commit files or groups?
- Tokens should definitely be per group in case we're uploading tons of small files

- todo: create meta ghosts
- todo: encode in upload token

# OIDC providers

Supported IdPs (Tier 1) plus Google and Auth0 as reference rows for non-homelab `sub` shapes. All entries support OIDC discovery.

| Provider             | `sub` format                         | `name` claim                                 |
| -------------------- | ------------------------------------ | -------------------------------------------- |
| Keycloak             | UUID v4 (`f5dab0e0-…`)               | yes (concat of first+last, if set)           |
| Authentik            | UUID v4                              | yes (full name field)                        |
| Authelia             | UUID v4                              | yes (`display_name` field)                   |
| Pocket-ID            | UUID v4                              | optional (user may not set it)               |
| Google _(reference)_ | numeric (`110248495921238986420`)    | yes (full name)                              |
| Auth0 _(reference)_  | `connection\|id` (e.g. `auth0\|abc`) | yes (often present; social connections vary) |

# Misc

```
rustup toolchain install nightly --allow-downgrade -c rustfmt
```
