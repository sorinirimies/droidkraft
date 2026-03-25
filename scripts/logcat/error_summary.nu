#!/usr/bin/env nu
# Error Summary — group errors by tag and show counts
# Usage: nu error_summary.nu logcat.jsonl

def main [file: path] {
    open $file
    | lines
    | where { |line| ($line | str length) > 0 }
    | each { |line| $line | from json }
    | where level in ["Error", "Fatal"]
    | group-by tag
    | transpose tag entries
    | each { |r| {
        tag: $r.tag,
        count: ($r.entries | length),
        sample: ($r.entries | first | get message | str substring 0..80)
    }}
    | sort-by count -r
}
