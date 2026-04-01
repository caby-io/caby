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
registration_enabled: false
paths:
  users_path: "users"
  spaces_path: "spaces"
spaces:
  - name: home
    archetype: users
    path: /some/other/path # optional override
    readonly: false
  - name: media
users:
  - name: caby_guy
    email: caby_guy@caby.io
    activation_token: OHQFhErYIM7xK8gMtf9emXt4LssVp5ibBs3MgJXTBQXbw8Cs4HUyWv1HdXjJyUL5
    spaces:
      - name: home
        permissions: "*"
```
