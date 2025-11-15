#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME=filler
SOLUTION_DIR="$(pwd)/solution"

if [ ! -d "$SOLUTION_DIR" ]; then
  echo "Error: solution directory not found at $SOLUTION_DIR"
  echo "Make sure you are running this script from the docker_image folder."
  exit 1
fi

echo "=== Building Docker image: $IMAGE_NAME ==="
docker build -t "$IMAGE_NAME" .

echo "=== Running game in Docker container ==="
docker run --rm -v "$SOLUTION_DIR":/filler/solution -it "$IMAGE_NAME" bash -lc '
  set -e

  echo "-> Building Rust bot..."
  cd /filler/solution
  cargo build --release

  echo "-> Starting game: P1 = your bot, P2 = bender"
  cd /filler
  ./linux_game_engine -f maps/map01 \
      -p1 solution/target/release/filler \
      -p2 linux_robots/bender
'
