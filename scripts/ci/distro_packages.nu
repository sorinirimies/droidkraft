#!/usr/bin/env nu
# Arch (.pkg.tar.zst) and Alpine (.apk) packages from the manifests render_packaging.nu
# wrote into dist/, built in Docker. Their sources are the release assets, so run this
# after the release is published.
# usage: nu scripts/ci/distro_packages.nu arch   [--dir dist] [--out pkgs]
#        nu scripts/ci/distro_packages.nu alpine [--dir dist] [--out pkgs]   (arm64 needs QEMU)

# The rendered PKGBUILD-<name> files (see render_packaging.nu outputs).
export const ARCH_PKGS = [droidkraft-tui-bin droidkraft-bin]

# Docker platform and Alpine architecture of each APKBUILD build.
export const ALPINE = [[platform arch]; [amd64 x86_64] [arm64 aarch64]]

# `docker` arguments that run makepkg on the PKGBUILD in `dir`, as a non-root user.
export def arch_args [dir: string]: nothing -> list<string> {
    let build = "pacman -Syu --noconfirm base-devel >/dev/null && useradd -m b && chown -R b /pkg && su b -c 'cd /pkg && makepkg -f --noconfirm --nodeps'"
    [run --rm -v $"($dir):/pkg" archlinux:latest bash -c $build]
}

# `docker` arguments that run abuild on the APKBUILD in `dir` and copy the .apk back.
export def alpine_args [dir: string, platform: string, arch: string]: nothing -> list<string> {
    let build = $"apk add --no-cache alpine-sdk >/dev/null && adduser -D b && addgroup b abuild && chown -R b /w && su b -c 'abuild-keygen -a -n && cd /w && abuild checksum && abuild -r' && cp /home/b/packages/w/($arch)/*.apk /w/"
    [run --rm --platform $"linux/($platform)" -v $"($dir):/w" alpine:3.20 sh -c $build]
}

# abuild names both architectures' packages alike; the release needs them apart.
export def apk_name [file: string, arch: string]: nothing -> string {
    $file | str replace --regex '\.apk$' $"-($arch).apk"
}

def main [] { print -e "usage: distro_packages.nu arch|alpine [--dir dist] [--out pkgs]" }

def "main arch" [--dir: string = "dist", --out: string = "pkgs"] {
    mkdir $out
    for p in $ARCH_PKGS {
        let work = $"arch/($p)" | path expand
        mkdir $work
        cp $"($dir)/PKGBUILD-($p)" $"($work)/PKGBUILD"
        ^docker ...(arch_args $work)
        for f in (glob $"($work)/*.pkg.tar.zst") { cp $f $out }
    }
}

def "main alpine" [--dir: string = "dist", --out: string = "pkgs"] {
    mkdir $out
    for a in $ALPINE {
        let work = $"alpine-($a.arch)" | path expand
        mkdir $work
        cp $"($dir)/APKBUILD" $"($work)/APKBUILD"
        ^docker ...(alpine_args $work $a.platform $a.arch)
        for f in (glob $"($work)/*.apk") { cp $f ($out | path join (apk_name ($f | path basename) $a.arch)) }
    }
}
