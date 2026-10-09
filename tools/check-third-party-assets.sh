#!/usr/bin/env bash
# Fails if any file in the given commits is byte-identical to an image, audio
# or CSV file from a reference directory (e.g. an extracted rekordbox installer).
#
# Usage: tools/check-third-party-assets.sh <reference-dir> [<rev-range>]
#   Without a range, checks the files tracked in HEAD.
#   Example before publishing: tools/check-third-party-assets.sh \
#       ~/Downloads/rekordbox_extracted 2.5.6..HEAD
set -euo pipefail

reference=${1:?usage: $0 <reference-dir> [<rev-range>]}
range=${2:-}

known=$(mktemp)
trap 'rm -f "$known"' EXIT
find "$reference" -type f \( -iname '*.svg' -o -iname '*.png' -o -iname '*.jpg' \
        -o -iname '*.wav' -o -iname '*.csv' -o -iname '*.onnx' -o -iname '*.pb' \) \
        -print0 | xargs -0 -r sha256sum | awk '{print $1}' | sort -u > "$known"

if [[ -n "$range" ]]; then
    commits=$(git rev-list "$range")
else
    commits=$(git rev-parse HEAD)
fi

found=0
# git blob ids are not SHA-256, so hash the content of every unique blob once.
while IFS=$'\t' read -r blob path; do
    hash=$(git cat-file blob "$blob" | sha256sum | awk '{print $1}')
    if grep -qx "$hash" "$known"; then
        echo "third-party file: $path ($blob)"
        found=1
    fi
done < <(for c in $commits; do
        git ls-tree -r "$c" | awk -F'\t' '{split($1, a, " "); print a[3] "\t" $2}'
    done | sort -u -t$'\t' -k1,1)

if (( found )); then
    echo "Third-party files found; do not publish." >&2
    exit 1
fi
echo "No third-party files found."
