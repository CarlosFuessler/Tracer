#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$script_dir"

profile="${PCB_EDITOR_PROFILE:-debug}"

case "${1-}" in
  --release)
    profile="release"
    shift
    ;;
  --debug)
    profile="debug"
    shift
    ;;
esac

if [[ "$profile" == "release" ]]; then
  cargo run --release --package schematic_editor -- "$@"
else
  cargo run --package schematic_editor -- "$@"
fi
