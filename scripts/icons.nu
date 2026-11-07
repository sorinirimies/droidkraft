#!/usr/bin/env nu
# ──────────────────────────────────────────────────────────────────────────────
# DroidKraft — App icons
# ──────────────────────────────────────────────────────────────────────────────
# One master artwork, assets/icons/<id>.svg, and every platform icon derived from
# it. Its marked blocks (<!-- construction:start --> … :end, likewise detail, rim
# and mark) let the variants drop or reshape parts:
#   full         the master as drawn (Linux, Windows, ≥ 48 px)
#   small        no construction lines, detail or rim, the mark enlarged: crisp
#                at 16–32 px where the fine lines would only blur
#   macos        the full art on Apple's 1024 grid: 824 px body, margin and shadow
#   macos-small  the same around the small variant
#
# Writes (all committed):
#   assets/icons/<id>-<n>.png        Linux hicolor sizes (ICON_SIZES)
#   packaging/macos/<id>.icns        the .app icon, 16 – 1024 px
#   packaging/windows/<id>.ico       installer and Start-menu icon
#   GUI_ASSETS/<id>.ico              embedded into the .exe by build.rs
#   GUI_ASSETS/<id>-128.rgba         the window icon (Windows, X11), when WINDOW_RGBA
#   GUI_ASSETS/<id>-macos-512.png    the Dock icon a bare binary sets on macOS, when DOCK_PNG
#
# Needs resvg and ImageMagick (`magick`); the .icns also needs macOS `iconutil`.
# Usage: nu scripts/icons.nu   (or: just icons)
# ──────────────────────────────────────────────────────────────────────────────

use project.nu [APP]
use ci/package_desktop.nu [ICON_SIZES]

export const MASTER = $"assets/icons/($APP.id).svg"
# The largest pixel size that gets a "small" variant.
export const SMALL_MAX = 32
# Sizes inside the Windows .ico: the shell picks 16–48, high-DPI up to 256.
export const ICO_SIZES = [16 20 24 32 40 48 64 96 128 256]
export const WINDOW_ICON_SIZE = 128
# The GUI crate's embedded assets; "" when the GUI embeds none.
export const GUI_ASSETS = ""
# Which window icons the GUI embeds: raw RGBA (Windows, X11) and the macOS Dock PNG.
export const WINDOW_RGBA = false
export const DOCK_PNG = false

# Apple's macOS icon grid: the body is 824 of 1024 px, centred.
const MAC_BODY = 824

# A block of the master between its <!-- name:start --> and <!-- name:end --> markers.
def block-regex [name: string]: nothing -> string {
    $'\s*<!-- ($name):start -->.*?<!-- ($name):end -->'
}

# The inside of an <svg> document: everything between its opening and closing tag.
def svg-body [svg: string]: nothing -> string {
    $svg | str replace --regex '(?s)^.*?<svg[^>]*>' '' | str replace --regex '(?s)</svg>\s*$' ''
}

# Wrap a 1024 px icon onto Apple's grid: scaled to the body size, centred, with
# the soft drop shadow macOS icons carry.
def on-mac-grid [svg: string]: nothing -> string {
    let scale = $MAC_BODY / 1024
    let offset = (1024 - $MAC_BODY) / 2
    '<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024">
  <defs>
    <filter id="mac-shadow" x="-10%" y="-10%" width="120%" height="125%">
      <feDropShadow dx="0" dy="12" stdDeviation="14" flood-color="#000000" flood-opacity="0.35"/>
    </filter>
  </defs>
  <g filter="url(#mac-shadow)">
    <g transform="translate(OFFSET OFFSET) scale(SCALE)">BODY</g>
  </g>
</svg>
'
    | str replace --all 'OFFSET' ($offset | into string)
    | str replace 'SCALE' ($scale | into string)
    | str replace 'BODY' (svg-body $svg)
}

# The SVG of one variant of the master (see the header).
export def variant [master: string, kind: string]: nothing -> string {
    match $kind {
        "full" => $master
        "small" => {
            ["construction" "detail" "rim"]
            | reduce --fold $master {|b, svg| $svg | str replace --all --regex $"\(?s\)(block-regex $b)" '' }
            | str replace --regex '<!-- mark:start -->' '<!-- mark:start --><g transform="translate(512 512) scale(1.12) translate(-512 -512)">'
            | str replace --regex '<!-- mark:end -->' '</g><!-- mark:end -->'
        }
        "macos" => (on-mac-grid $master)
        "macos-small" => (on-mac-grid (variant $master small))
        _ => { error make {msg: $"unknown icon variant: ($kind)"} }
    }
}

# Which variant a raster of `px` pixels uses, on the Mac grid or not.
export def kind-for [px: int, mac: bool]: nothing -> string {
    let small = $px <= $SMALL_MAX
    match [$mac $small] {
        [true true] => "macos-small"
        [true false] => "macos"
        [false true] => "small"
        _ => "full"
    }
}

# The files of a macOS .iconset: {name, px}, 16 pt to 512 pt at 1x and 2x.
export def iconset-entries []: nothing -> list<record<name: string, px: int>> {
    [16 32 128 256 512] | each {|pt|
        [{name: $"icon_($pt)x($pt).png", px: $pt} {name: $"icon_($pt)x($pt)@2x.png", px: ($pt * 2)}]
    } | flatten
}

# Every file `main` writes, relative to the repository root.
export def outputs []: nothing -> list<string> {
    let id = $APP.id
    let pngs = $ICON_SIZES | each {|n| $"assets/icons/($id)-($n).png" }
    let window = (
        (if $WINDOW_RGBA { [$"($GUI_ASSETS)/($id)-($WINDOW_ICON_SIZE).rgba"] } else { [] })
        ++ (if $DOCK_PNG { [$"($GUI_ASSETS)/($id)-macos-512.png"] } else { [] })
    )
    let gui = if ($GUI_ASSETS | is-empty) { [] } else { [$"($GUI_ASSETS)/($id).ico"] ++ $window }
    $pngs ++ [
        $"packaging/macos/($id).icns"
        $"packaging/windows/($id).ico"
    ] ++ $gui
}

def need [tool: string, hint: string] {
    if (which $tool | is-empty) { error make {msg: $"($tool) not found: ($hint)"} }
}

# Rasterise variant `kind` at `px` × `px` into `out`.
def render [variants: record, kind: string, px: int, out: string] {
    ^resvg --width $px --height $px ($variants | get $kind) $out
}

def main [] {
    let root = $env.FILE_PWD | path dirname
    cd $root
    need resvg "brew install resvg  /  cargo install resvg"
    need magick "brew install imagemagick  /  apt install imagemagick"
    let id = $APP.id

    let tmp = mktemp --directory
    let master = open --raw $MASTER
    let variants = ["full" "small" "macos" "macos-small"] | reduce --fold {} {|k, acc|
        let path = $"($tmp)/($k).svg"
        variant $master $k | save --force $path
        $acc | insert $k $path
    }

    # Linux hicolor PNGs
    for n in $ICON_SIZES { render $variants (kind-for $n false) $n $"assets/icons/($id)-($n).png" }

    # Windows .ico: uncompressed BMP entries, which NSIS and every Windows version read
    let ico_pngs = $ICO_SIZES | each {|n|
        let p = $"($tmp)/ico-($n).png"
        render $variants (kind-for $n false) $n $p
        $p
    }
    mkdir packaging/windows
    ^magick ...$ico_pngs -define icon:format=bmp $"packaging/windows/($id).ico"
    if ($GUI_ASSETS | is-not-empty) {
        mkdir $GUI_ASSETS
        cp $"packaging/windows/($id).ico" $"($GUI_ASSETS)/($id).ico"
    }

    # Window icon (Windows title bar and taskbar, X11)
    if $WINDOW_RGBA {
        let win_png = $"($tmp)/window.png"
        render $variants (kind-for $WINDOW_ICON_SIZE false) $WINDOW_ICON_SIZE $win_png
        ^magick $win_png -depth 8 $"RGBA:($GUI_ASSETS)/($id)-($WINDOW_ICON_SIZE).rgba"
    }
    if $DOCK_PNG { render $variants macos 512 $"($GUI_ASSETS)/($id)-macos-512.png" }

    # macOS .icns
    if (which iconutil | is-empty) {
        print -e $"⚠️  iconutil not found \(macOS only\): packaging/macos/($id).icns left as it is"
    } else {
        let set = $"($tmp)/($id).iconset"
        mkdir $set
        for e in (iconset-entries) { render $variants (kind-for $e.px true) $e.px $"($set)/($e.name)" }
        mkdir packaging/macos
        ^iconutil --convert icns --output $"packaging/macos/($id).icns" $set
    }

    rm --recursive $tmp
    print $"✅ icons written:\n(outputs | each {|f| $'  ($f)' } | str join "\n")"
}
