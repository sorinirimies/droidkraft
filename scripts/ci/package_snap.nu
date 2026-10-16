#!/usr/bin/env nu
# Stage a snapcraft project from the prebuilt binaries (build it with snapcore/action-build, path: snapbuild).
# usage: nu scripts/ci/package_snap.nu <version>   -> snapbuild/{snap/snapcraft.yaml,stage/*}
def main [version: string] {
    rm -rf snapbuild
    mkdir snapbuild/snap snapbuild/stage
    cp target/release/droidkraft target/release/droidkraft-tui assets/icons/droidkraft.png snapbuild/stage/
    open --raw packaging/linux/droidkraft.desktop
    | str replace --regex '(?m)^Icon=.*' 'Icon=${SNAP}/usr/share/icons/hicolor/256x256/apps/droidkraft.png'
    | save -f snapbuild/stage/droidkraft.desktop
    open --raw packaging/snap/snapcraft.yaml.in | str replace --all "@VERSION@" $version | save -f snapbuild/snap/snapcraft.yaml
}
