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
    ├── init.yaml     # temporary file that facilitates user init
    ├── password      # user's salted & hashed p/w
    ├── profile.yaml  # user's profile, config, & preferences info
    ├── session_...   # an active user session
    └── ...more
```

### init.yaml

```yaml
method: code # code|email
locked_until: 2025-12-15T02:59:43.1Z # optional to prevent brute force
# only one of these should be active at a time
# this code is printed into stdout
code:
  value: 000000 # 6-digit code
  attempts: 3 # max 3 attempts before the code is reset
  created_at: 2025-12-15T02:59:43.1Z # valid for 24 hours

# an email is sent with an activation link
email:
  value: "todo: uuid" # uuid
  attempts: 3 # max 3 attempts before the uuid is reset
  created_at: 2025-12-15T02:59:43.1Z # valid for 24 hrs
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
    spaces:
      - name: home
        permissions: "*"
```
