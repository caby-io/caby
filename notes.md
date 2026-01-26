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
  ├── configs
  ├── users
  └── spaces
    ├── user
      ├── live
      └── meta
    └── media
      ├── live
      └── meta
```

# Request Paths

/list/{space}/

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
    archetype: users
    path: /some/other/path
    readonly: false
  - name: media
```
