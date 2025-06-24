#!/bin/bash

set -euo pipefail

# go to service dir
pushd $(dirname ${0})
cd ../

# load env vars
set -a; source .env; set +a

cargo watch -q -c -w src/ -x run
