#!/usr/bin/env nu
# ──────────────────────────────────────────────────────────────────────────────
# DroidKraft — Keep binaries only on the newest release
# ──────────────────────────────────────────────────────────────────────────────
# Deletes the assets of every release except <keep>. Release notes and tags stay.
# Usage:
#   nu scripts/ci/prune_releases.nu <keep> [--dry-run]                    GitHub (gh + GH_TOKEN)
#   nu scripts/ci/prune_releases.nu <keep> --gitea-api <server>/api/v1/repos/<owner>/<repo>
#                                                                          Gitea (GITEA_TOKEN)
# ──────────────────────────────────────────────────────────────────────────────

# `gh release view --json tagName,assets` records → { tag, assets: [{ name, id }] }.
export def from_gh [releases: list]: nothing -> list {
    $releases | each {|r| { tag: $r.tagName, assets: ($r.assets? | default [] | each {|a| { name: $a.name, id: $a.name } }) } }
}

# Gitea `GET /releases` records → { tag, release, assets: [{ name, id }] }.
export def from_gitea [releases: list]: nothing -> list {
    $releases | each {|r| { tag: $r.tag_name, release: $r.id, assets: ($r.assets? | default [] | each {|a| { name: $a.name, id: $a.id } }) } }
}

# The assets to delete, one record per asset. Refuses when <keep> is unknown or has
# no assets, so a typo or a failed upload can't leave no binaries anywhere.
export def doomed [releases: list, keep: string]: nothing -> list {
    let kept = $releases | where tag == $keep
    if ($kept | is-empty) {
        error make --unspanned { msg: $"release ($keep) not found; refusing to prune" }
    }
    if ($kept.0.assets | is-empty) {
        error make --unspanned { msg: $"release ($keep) has no assets; refusing to prune" }
    }
    $releases | where tag != $keep
    | each {|r| $r.assets | each {|a| { tag: $r.tag, release: ($r.release? | default $r.tag), name: $a.name, id: $a.id } } }
    | flatten
}

def gitea_releases [api: string, headers: record]: nothing -> list {
    mut all = []
    mut page = 1
    loop {
        let batch = http get --headers $headers $"($api)/releases?limit=50&page=($page)"
        if ($batch | is-empty) { break }
        $all = $all ++ $batch
        $page += 1
    }
    $all
}

def main [keep: string, --gitea-api: string, --dry-run] {
    if $gitea_api != null {
        let headers = { Authorization: $"token ($env.GITEA_TOKEN)" }
        for a in (doomed (from_gitea (gitea_releases $gitea_api $headers)) $keep) {
            print -e $"delete ($a.tag)/($a.name)"
            if not $dry_run { http delete --headers $headers $"($gitea_api)/releases/($a.release)/assets/($a.id)" | ignore }
        }
        return
    }
    let releases = ^gh release list --limit 200 --json tagName | from json
        | each {|r| ^gh release view $r.tagName --json tagName,assets | from json }
    for a in (doomed (from_gh $releases) $keep) {
        print -e $"delete ($a.tag)/($a.name)"
        if not $dry_run { ^gh release delete-asset $a.tag $a.name --yes }
    }
}
