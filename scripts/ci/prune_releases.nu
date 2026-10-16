#!/usr/bin/env nu
# Delete the assets of every GitHub release except --keep (notes + tags stay).
# --whole deletes the old releases (and their tags) entirely.
# usage: nu scripts/ci/prune_releases.nu v1.2.0 [--dry-run] [--whole]     (needs gh + GH_TOKEN)

# Which (tag, asset) pairs to delete. Refuses when `keep` is unknown, so a typo can't wipe everything.
export def doomed [releases: list, keep: string]: nothing -> list {
    if ($releases | where tagName == $keep | is-empty) {
        error make --unspanned { msg: $"release ($keep) not found; refusing to prune" }
    }
    $releases | where tagName != $keep
    | each {|r| ($r.assets? | default [] | each {|a| { tag: $r.tagName, asset: $a.name } }) }
    | flatten
}

def main [keep: string, --dry-run, --whole] {
    let releases = (^gh release list --limit 200 --json tagName | from json
        | each {|r| ^gh release view $r.tagName --json tagName,assets | from json })
    if $whole {
        for r in ($releases | where tagName != $keep) {
            print $"delete release ($r.tagName)"
            if not $dry_run { ^gh release delete $r.tagName --yes --cleanup-tag }
        }
        return
    }
    for a in (doomed $releases $keep) {
        print $"delete ($a.tag)/($a.asset)"
        if not $dry_run { ^gh release delete-asset $a.tag $a.asset --yes }
    }
}
