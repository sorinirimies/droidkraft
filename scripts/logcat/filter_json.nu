#!/usr/bin/env nu
# Filter JSON — extract and pretty-print JSON payloads from log messages
# Usage: nu filter_json.nu logcat.jsonl

def main [file: path] {
    open $file
    | lines
    | where { |line| ($line | str length) > 0 }
    | each { |line| $line | from json }
    | where { |r| ($r.message | str contains "{") and ($r.message | str contains "}") }
    | each { |r|
        let json_start = ($r.message | str index-of "{")
        let json_part = ($r.message | str substring $json_start..)
        try {
            let parsed = ($json_part | from json)
            {
                timestamp: $r.timestamp,
                tag: $r.tag,
                json: ($parsed | to json --indent 2)
            }
        } catch {
            null
        }
    }
    | where { |r| $r != null }
}
