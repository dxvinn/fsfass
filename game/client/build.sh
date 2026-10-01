#!/usr/bin/env sh
# Build the Rust simulation bridge and copy it into the Godot project.
set -e
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
cargo build --release -p genesis-gdext
for f in libgenesis_gdext.so genesis_gdext.dll libgenesis_gdext.dylib; do
  [ -f "target/release/$f" ] && cp "target/release/$f" game/client/bin/
done
echo "Built. Run: godot --path game/client"
