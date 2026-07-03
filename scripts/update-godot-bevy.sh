#!/usr/bin/env bash
# Vendor the `addons/godot-bevy` subfolder from the upstream godot-bevy repo
# into project-template/addons/godot-bevy, without pulling the rest of the repo.
#
# Re-run this script to update to a newer upstream revision.
# Usage: scripts/update-godot-bevy.sh [git-ref]   (default ref: main)
set -euo pipefail

REPO="https://github.com/bytemeadow/godot-bevy"
SRC_SUBDIR="addons/godot-bevy"
REF="${1:-main}"

# Resolve paths relative to the repo root, regardless of where we're invoked from.
ROOT="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
DEST="$ROOT/godot/project-template/addons/godot-bevy"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "Fetching $SRC_SUBDIR from $REPO@$REF ..."
git clone --depth=1 --filter=blob:none --sparse --branch "$REF" "$REPO" "$TMP"
git -C "$TMP" sparse-checkout set "$SRC_SUBDIR"

PINNED_SHA="$(git -C "$TMP" rev-parse HEAD)"

mkdir -p "$DEST"
rsync -a --delete --exclude='.git' "$TMP/$SRC_SUBDIR/" "$DEST/"

# Record exactly which upstream commit these files came from.
printf '%s %s\n' "$PINNED_SHA" "$REF" > "$DEST/.upstream-revision"

echo "Vendored godot-bevy @ $PINNED_SHA ($REF) -> $DEST"
