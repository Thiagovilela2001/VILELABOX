#!/usr/bin/env bash
set -euo pipefail
rbx_source_dir="$(cd -- "$(dirname -- "$0")" && pwd)"
rbx_settings_dir="$(dirname -- "$rbx_source_dir")/bars-profile"
if [[ ! -x "$rbx_source_dir/build/mixxx" ]]; then
  echo 'Compile primeiro: bash ~/.mixxx/mixxx-source/build-bars.sh' >&2
  exit 1
fi
python3 "$rbx_source_dir/prepare-bars-profile.py" "$rbx_settings_dir"
exec "$rbx_source_dir/build/mixxx" --settings-path "$rbx_settings_dir" --resource-path "$rbx_source_dir/res" "$@"
