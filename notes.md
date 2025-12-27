# Files response

```json
{
  "name": "Name of the entry",
  "path": "relative/path/of/entry",
  ...
}
```

# Dir structure

caby-home

- /configs
- /users
- /spaces
  - /user
    - /live
    - /meta
  - /media
    - /live
    - /meta

# Configs

```yaml
# server.yaml
---
registration_enabled: false
paths:
  users_path: "users"
  spaces_path: "spaces"
spaces:
  - name: home
  - name: media
```
