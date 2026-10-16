#!/usr/bin/env nu
# Render the AUR PKGBUILDs and Homebrew formula from the release tarballs.
# usage: nu scripts/ci/render_packaging.nu <version> [--dir dist]
# Writes dist/PKGBUILD-droidkraft-tui-bin, dist/PKGBUILD-droidkraft-bin, dist/droidkraft-tui.rb

# sha256 of `<app>-<version>-<target>.tar.gz` in `dir`, or "SKIP" when that target was not built.
export def sha [dir: string, version: string, target: string, app: string = "droidkraft-tui"]: nothing -> string {
    let f = $"($dir)/($app)-($version)-($target).tar.gz"
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
        SHA_GUI_X86_64: (sha $dir $version "x86_64-unknown-linux-gnu" "droidkraft")
    }
    mkdir $dir
    render (open --raw packaging/aur/PKGBUILD.in) $vars | save -f $"($dir)/PKGBUILD-droidkraft-tui-bin"
    render (open --raw packaging/aur/PKGBUILD-gui.in) $vars | save -f $"($dir)/PKGBUILD-droidkraft-bin"
    render (open --raw packaging/homebrew/droidkraft-tui.rb.in) $vars | save -f $"($dir)/droidkraft-tui.rb"
    render (open --raw packaging/homebrew/droidkraft.rb.in) $vars | save -f $"($dir)/droidkraft.rb"
    print $"rendered PKGBUILDs and formula into ($dir) for ($version)"
}
