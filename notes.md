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
    path: /some/other/path
    readonly: false
  - name: media
```
