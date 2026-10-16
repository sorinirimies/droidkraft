#!/usr/bin/env bash
# usage: package_flatpak.sh <version>   -> dist/droidkraft-<version>.flatpak
set -euo pipefail
V="$1"; ID=io.github.sorinirimies.DroidKraft; mkdir -p dist
rm -rf fp && mkdir -p fp/stage && cd fp
cp ../target/release/droidkraft ../target/release/droidkraft-tui stage/
cp ../assets/icons/droidkraft.png stage/
sed "s|^Icon=.*|Icon=$ID|;s|^Exec=.*|Exec=droidkraft|" ../packaging/linux/droidkraft.desktop > stage/$ID.desktop
sed "s|VERSION|$V|;s|DATE|$(date +%F)|" ../packaging/linux/$ID.metainfo.xml > stage/$ID.metainfo.xml
cp ../packaging/flatpak/$ID.yml .
flatpak-builder --force-clean --user --disable-rofiles-fuse --repo=repo build $ID.yml
flatpak build-bundle repo "../dist/droidkraft-$V.flatpak" $ID
