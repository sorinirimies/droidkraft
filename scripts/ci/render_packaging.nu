#!/usr/bin/env nu
# Render the AUR PKGBUILD and Homebrew formula from the release tarballs.
# usage: nu scripts/ci/render_packaging.nu <version> [--dir dist]
# Reads dist/droidkraft-tui-<v>-<target>.tar.gz checksums, writes dist/PKGBUILD and dist/droidkraft-tui.rb.

# sha256 of a release tarball, or a placeholder when that target was not built.
export def sha [dir: string, version: string, target: string]: nothing -> string {
    let f = $"($dir)/droidkraft-tui-($version)-($target).tar.gz"
    if ($f | path exists) { open --raw $f | hash sha256 } else { "SKIP" }
}

export def render [tpl: string, vars: record]: nothing -> string {
    $vars | items {|k, v| [$k $v] } | reduce --fold $tpl {|it, acc| $acc | str replace --all $"@($it.0)@" $it.1 }
}

def main [version: string, --dir: string = "dist"] {
    let vars = {
        VERSION: $version
        SHA_X86_64: (sha $dir $version "x86_64-unknown-linux-gnu")
        SHA_AARCH64: (sha $dir $version "aarch64-unknown-linux-gnu")
    }
    mkdir $dir
    render (open --raw packaging/aur/PKGBUILD.in) $vars | save -f $"($dir)/PKGBUILD"
    render (open --raw packaging/homebrew/droidkraft-tui.rb.in) $vars | save -f $"($dir)/droidkraft-tui.rb"
    print $"rendered ($dir)/PKGBUILD and ($dir)/droidkraft-tui.rb for ($version)"
}
