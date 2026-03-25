#!/usr/bin/env nu
# Timeline — show log volume per second
# Usage: nu timeline.nu logcat.jsonl

def main [file: path] {
    open $file
    | lines
    | where { |line| ($line | str length) > 0 }
    | each { |line| $line | from json }
    | where timestamp != null
    | each { |r| { second: ($r.timestamp | str substring 0..8), level: $r.level } }
    | group-by second
    | transpose second entries
    | each { |r| {
        time: $r.second,
        total: ($r.entries | length),
        errors: ($r.entries | where level in ["Error", "Fatal"] | length),
    }}
    | sort-by time
}
