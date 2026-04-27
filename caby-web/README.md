# caby-web

Frontend for Caby — self-hosted file management

### ⚠️ Note: Caby is in a pre-release state and its components are still a work-in-progress. Please provide feedback so we can resolve any issues.

## Environment Variables

| Variable          | Default                    | Description               |
| ----------------- | -------------------------- | ------------------------- |
| `PUBLIC_API_BASE` | `http://localhost:8080/v0` | Caby backend API base URL |
| `PORT`            | `3000`                     | Listen port               |

## Run

```sh
docker run -p 3000:3000 \
  -e PUBLIC_API_BASE=http://localhost:8080/v0 \
  cabynet/caby-web:latest
```
