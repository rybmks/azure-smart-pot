#!/usr/bin/env bash

docker run -it --rm \
    --network smart-pot \
    --env-file .env \
    --name azure-server \
    bash -c "cargo run"