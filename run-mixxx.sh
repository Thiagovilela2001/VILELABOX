#!/usr/bin/env bash
# blackmesa: abre o Mixxx com o mod de compassos usando o perfil principal (~/.mixxx).
set -euo pipefail
rbx_source_dir="$(cd -- "$(dirname -- "$0")" && pwd)"
if [[ ! -x "$rbx_source_dir/build/mixxx" ]]; then
  echo 'Compile primeiro: bash ~/.mixxx/mixxx-source/build-bars.sh' >&2
  exit 1
fi
config="$HOME/.mixxx/mixxx.cfg"
if [[ -f "$config" ]]; then
  sed -i -e 's/^ResizableSkin RbxFlow$/ResizableSkin RbxFlowBars/' \
         -e '/^ResizableSkin=RbxFlow *$/d' "$config"
fi
exec "$rbx_source_dir/build/mixxx" --resource-path "$rbx_source_dir/res" "$@"
