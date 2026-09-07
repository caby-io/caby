<p align="center">
  <picture>
    <source srcset=".github/assets/logo-white.png" media="(prefers-color-scheme: dark)">
    <img src=".github/assets/logo-black.png" alt="Caby Logo" width="300">
  </picture>
</p>

<p align="center">
  <em><ins>Simple and reliable</ins> self-hosted file management for your home network.</em>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-AGPL_v3-blue.svg" alt="License: AGPL v3"></a>
  <a href="https://discord.gg/Z2JkSs2Hzy"><img src="https://img.shields.io/badge/Discord-Join-5865F2?logo=discord&logoColor=white" alt="Discord"></a>
</p>

<p align="center">
  <img src=".github/assets/preview.gif" alt="Caby file browser" width="860">
</p>

## ✨ Features

- Requires **no backing services**. Everything is managed by the backend runtime.
- Files all the way down: Everything from configuration to metadata is stored in readable files.
- Integrate with your favorite **OIDC** provider: Authentik, Pocket ID, Authelia, etc. Or, use the
  built-in password auth.
- Organize your files within **spaces** for compartmentalization and easy access control
- Files are uploaded using **chunked uploads** for resumability, performance, and compatibility with
  certain ingress providers.
- Share files, photos, and videos with family or guests using **shares**.
- Supports ARM images for Raspberry Pi and other lightweight devices.

## 🗺️ Roadmap

What we're working towards in the near future:

- [ ] File shares _(in progress)_
  - [x] File shares MVP
  - [ ] Guest uploads, writes, and deletes
- [ ] Background tasks (download files, convert videos, and more) _(in progress)_
  - [x] Background jobs system
  - [ ] Websocket events
  - [ ] Background downloads
- [ ] Web file editor
- [ ] File versioning
- [ ] Fine-grained user access
- [ ] Device syncing

Are we missing something? Let us know on [Discord](https://discord.gg/Z2JkSs2Hzy) or
[open an issue](https://github.com/caby-io/caby/issues).

🚧 Caby is pre-1.0 and under active development. Occasional breaking changes to **config and APIs**
may require human input between versions. Caby stores your files plainly and they should never be
affected by Caby version upgrades.

## 🚀 Quick Start

For more complete installation and configuration information please refer to
[Caby's documentation](https://caby.io/getting-started/).

### Docker Compose

1. Grab the starter `compose.yaml` and `config.yaml`:

   ```bash
   curl -fO https://raw.githubusercontent.com/caby-io/caby/main/docker/compose.yaml &&
   curl -fO https://raw.githubusercontent.com/caby-io/caby/main/docker/config.yaml
   ```

2. Generate a 64-character activation token and replace `REPLACE_ME` in `config.yaml`, or, replace
   the config's token in one shot:

   ```bash
   # Unix
   sed -i "s/REPLACE_ME/$(openssl rand -hex 32)/" config.yaml
   ```

   ```bash
   # macOS
   sed -i '' "s/REPLACE_ME/$(openssl rand -hex 32)/" config.yaml
   ```

   Make sure you note the activation token for the final step.

3. Deploy:

   ```bash
   docker compose up -d
   ```

4. Navigate to the activation page (e.g. http://localhost:3000/activate) to activate `caby_user` and
   set your password.

For bare `docker run`, reverse-proxy setup, and full configuration options, see the
[Docker installation guide](https://caby.io/installation/docker/).

### Helm / Kubernetes

Caby ships an official Helm chart.

1. Grab the example `config.yaml`:

   ```bash
   curl -fO https://raw.githubusercontent.com/caby-io/caby/main/examples/config.yaml
   ```

2. Generate a 64-character activation token and replace `REPLACE_ME` in `config.yaml`, or, replace
   the config's token in one shot:

   ```bash
   # Unix
   sed -i "s/REPLACE_ME/$(openssl rand -hex 32)/" config.yaml
   ```

   ```bash
   # macOS
   sed -i '' "s/REPLACE_ME/$(openssl rand -hex 32)/" config.yaml
   ```

   Make sure you note the activation token for the final step.

3. Create a helm release with `--set ingress.web.host=` to your desired domain:

   ```bash
   helm install caby oci://ghcr.io/caby-io/charts/caby \
     --namespace caby --create-namespace \
     --set ingress.web.host=files.example.com \
     --set-file config.inline=./config.yaml
   ```

4. Once the pods are up, navigate to the activation page on your ingress host (e.g.
   https://files.example.com/activate) to activate `caby_user` and set your password.

For the full `config.yaml` format and all chart options, see the
[Helm installation guide](https://caby.io/installation/helm/).
