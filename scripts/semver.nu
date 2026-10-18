#!/usr/bin/env nu
# SemVer comparison helpers shared by bump_version.nu and its tests.

# Compare two prerelease identifiers (dot separated), per SemVer §11.
def pre-cmp [a: string, b: string] {
    let pa = ($a | split row ".")
    let pb = ($b | split row ".")
    let n = ([($pa | length) ($pb | length)] | math min)
    for i in 0..<$n {
        let x = ($pa | get $i)
        let y = ($pb | get $i)
        let xn = ($x =~ '^[0-9]+$')
        let yn = ($y =~ '^[0-9]+$')
        let c = if $xn and $yn {
            let xi = ($x | into int)
            let yi = ($y | into int)
            if $xi < $yi { -1 } else if $xi > $yi { 1 } else { 0 }
        } else if $xn { -1 } else if $yn { 1 } else {
            if $x < $y { -1 } else if $x > $y { 1 } else { 0 }
        }
        if $c != 0 { return $c }
    }
    let la = ($pa | length)
    let lb = ($pb | length)
    if $la < $lb { -1 } else if $la > $lb { 1 } else { 0 }
}

# Returns -1 / 0 / 1 when `a` is lower than / equal to / higher than `b`.
# Accepts X.Y.Z or X.Y.Z-prerelease. A release outranks its own prereleases.
export def version-cmp [a: string, b: string] {
    let pa = ($a | parse --regex '^(?P<core>[0-9]+\.[0-9]+\.[0-9]+)(?:-(?P<pre>.+))?$' | first)
    let pb = ($b | parse --regex '^(?P<core>[0-9]+\.[0-9]+\.[0-9]+)(?:-(?P<pre>.+))?$' | first)
    let ca = ($pa.core | split row "." | each { into int })
    let cb = ($pb.core | split row "." | each { into int })
    for i in 0..2 {
        let x = ($ca | get $i)
        let y = ($cb | get $i)
        if $x < $y { return (-1) }
        if $x > $y { return 1 }
    }
    let ha = ($pa.pre | is-not-empty)
    let hb = ($pb.pre | is-not-empty)
    if (not $ha) and (not $hb) { return 0 }
    if (not $ha) { return 1 }
    if (not $hb) { return (-1) }
    pre-cmp $pa.pre $pb.pre
}
