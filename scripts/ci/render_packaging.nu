#!/usr/bin/env nu
# Render package-manager manifests from the release artifacts in `dir`.
# usage: nu scripts/ci/render_packaging.nu <version> [--dir dist]
# Writes into dist/: PKGBUILD-droidkraft-tui-bin, PKGBUILD-droidkraft-bin, APKBUILD,
#   droidkraft-tui.rb, droidkraft.rb, droidkraft-tui.scoop.json, winget-*.yaml,
#   droidkraft-tui.nuspec, chocolateyinstall.ps1

# sha256 of `<app>-<version>-<target>.<ext>` in `dir`, or "SKIP" when that artifact was not built.
export def sha [dir: string, version: string, target: string, app: string = "droidkraft-tui", ext: string = "tar.gz"]: nothing -> string {
    let f = $"($dir)/($app)-($version)-($target).($ext)"
    if ($f | path exists) { open --raw $f | hash sha256 } else { "SKIP" }
}

export def render [tpl: string, vars: record]: nothing -> string {
    $vars | items {|k, v| [$k $v] } | reduce --fold $tpl {|it, acc| $acc | str replace --all $"@($it.0)@" $it.1 }
}

# Template -> output file pairs.
export def outputs []: nothing -> list<record<tpl: string, out: string>> {
    [
        [packaging/aur/PKGBUILD.in PKGBUILD-droidkraft-tui-bin]
        [packaging/aur/PKGBUILD-gui.in PKGBUILD-droidkraft-bin]
        [packaging/alpine/APKBUILD.in APKBUILD]
        [packaging/homebrew/droidkraft-tui.rb.in droidkraft-tui.rb]
        [packaging/homebrew/droidkraft.rb.in droidkraft.rb]
        [packaging/windows/droidkraft-tui.scoop.json.in droidkraft-tui.scoop.json]
        [packaging/windows/winget.version.yaml.in winget-sorinirimies.DroidKraftTui.yaml]
        [packaging/windows/winget.installer.yaml.in winget-sorinirimies.DroidKraftTui.installer.yaml]
        [packaging/windows/winget.locale.yaml.in winget-sorinirimies.DroidKraftTui.locale.en-US.yaml]
        [packaging/windows/droidkraft-tui.nuspec.in droidkraft-tui.nuspec]
        [packaging/windows/chocolateyinstall.ps1.in chocolateyinstall.ps1]
    ] | each {|p| { tpl: $p.0, out: $p.1 } }
}

def main [version: string, --dir: string = "dist"] {
    let vars = {
        VERSION: $version
        SHA_X86_64: (sha $dir $version "x86_64-unknown-linux-gnu")
        SHA_AARCH64: (sha $dir $version "aarch64-unknown-linux-gnu")
        SHA_GUI_X86_64: (sha $dir $version "x86_64-unknown-linux-gnu" "droidkraft")
        SHA_MACOS_ARM: (sha $dir $version "aarch64-apple-darwin")
        SHA_MACOS_X86: (sha $dir $version "x86_64-apple-darwin")
        SHA_WIN_ZIP: (sha $dir $version "x86_64-pc-windows-msvc" "droidkraft-tui" "zip")
    }
    mkdir $dir
    for o in (outputs) {
        render (open --raw $o.tpl) $vars | save -f $"($dir)/($o.out)"
    }
    print $"rendered (outputs | length) package manifests into ($dir) for ($version)"
}
