#!/usr/bin/env nu
# Find Crashes — detect FATAL exceptions and ANR patterns
# Usage: nu find_crashes.nu logcat.jsonl

def main [file: path] {
    let entries = (open $file
    | lines
    | where { |line| ($line | str length) > 0 }
    | each { |line| $line | from json })

    let fatals = ($entries | where level == "Fatal")
    let anrs = ($entries | where { |r| $r.message | str contains "ANR" })
    let exceptions = ($entries | where { |r|
        ($r.message | str contains "Exception")
        or ($r.message | str contains "FATAL")
    })

    print $"(ansi red_bold)Crashes & ANRs(ansi reset)"
    print $"  Fatal entries:     ($fatals | length)"
    print $"  ANR mentions:      ($anrs | length)"
    print $"  Exception entries: ($exceptions | length)"
    print ""

    if ($fatals | length) > 0 {
        print $"(ansi red)── Fatal Entries ──(ansi reset)"
        $fatals | select timestamp tag message | first 10
    }

    if ($anrs | length) > 0 {
        print $"(ansi yellow)── ANR Entries ──(ansi reset)"
        $anrs | select timestamp tag message | first 10
    }
}
