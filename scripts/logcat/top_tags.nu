#!/usr/bin/env nu
# Top Tags — rank logcat tags by frequency
# Usage: open logcat.jsonl | lines | each { from json } | source top_tags.nu
#    or: nu top_tags.nu logcat.jsonl

def main [file: path] {
    open $file
    | lines
    | where { |line| ($line | str length) > 0 }
    | each { |line| $line | from json }
    | where tag != null
    | group-by tag
    | transpose tag entries
    | each { |r| { tag: $r.tag, count: ($r.entries | length) } }
    | sort-by count -r
    | first 20
}
