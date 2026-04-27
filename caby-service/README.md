# caby-service

### ⚠️ Note: Caby is in a pre-release state and its components are still a work-in-progress. Please provide feedback so we can resolve any issues.

Backend API for Caby — self-hosted file management

## Deployment

Before your first deployment of the Caby service, create a config file:

```yaml
# config.yaml

# Spaces are compartments of storage to assist with
# organization and with controlling access
spaces:
  # the 'id' of the space. by default will correspond to
  # the path on the filesystem
  - name: home
    # how the space's name is displayed in Caby's UI
    display: 🏠 Home
  - name: media

# Caby's users
users:
  - name: caby-guy
    email: caby-guy@caby.io # optional
    # a 64-character string that will be used
    # to activate the user's account
    activation_token: <64-character string>
    # the user's access to spaces
    spaces:
      - name: home
        permissions: "*" # WIP: doesn't do anything yet
```

## Environment Variables

| Variable                       | Default        | Description                 |
| ------------------------------ | -------------- | --------------------------- |
| `CABY_HOME_PATH`               | `/app/cabynet` | Root storage path           |
| `CABY_DIRECTORY_META_FILENAME` | `.cabydir`     | Directory metadata filename |

## Run

```sh
docker run -p 8080:8080 \
  -v $(pwd)/cabynet:/app/cabynet \
  -v $(pwd)/config.yaml:/app/cabynet/config.yaml \
  cabynet/caby-service:latest
```
