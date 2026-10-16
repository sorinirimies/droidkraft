#!/usr/bin/env bash
# Delete the assets of every GitHub release except <keep-tag> (notes + tags stay).
# Set PRUNE_RELEASES=1 to delete the whole old releases (not just their files).
# usage: prune_releases.sh <keep-tag> [--dry-run]     (needs gh + GH_TOKEN)
set -euo pipefail
KEEP="$1"; DRY="${2:-}"
gh release view "$KEEP" >/dev/null || { echo "❌ release $KEEP not found; refusing to prune"; exit 1; }
for tag in $(gh release list --limit 200 --json tagName -q '.[].tagName'); do
  [ "$tag" = "$KEEP" ] && continue
  if [ "${PRUNE_RELEASES:-0}" = 1 ]; then
    echo "delete release $tag"; [ "$DRY" = --dry-run ] || gh release delete "$tag" --yes --cleanup-tag
    continue
  fi
  for a in $(gh release view "$tag" --json assets -q '.assets[].name'); do
    echo "delete $tag/$a"; [ "$DRY" = --dry-run ] || gh release delete-asset "$tag" "$a" --yes
  done
done
