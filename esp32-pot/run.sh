#!/usr/bin/env bash
docker network create --driver bridge smart-pot || true

docker run -it --rm \
  --network smart-pot \
  --device=/dev/ttyUSB0 \
  --group-add "$(stat -c "%g" /dev/ttyUSB0)" \
  --name esp32-pot \
  bash -c "cargo run"