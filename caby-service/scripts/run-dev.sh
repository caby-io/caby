#!/bin/bash

set -euo pipefail

# go to service dir
pushd $(dirname ${0})
cd ../

# load env vars
set -a; source ./configs/.dev.env; set +a

go run cmd/caby_api/*.go
