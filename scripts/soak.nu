#!/usr/bin/env nu
# Sample a running process's memory (RSS) and report growth.
#   nu scripts/soak.nu droidkraft-tui 120     # start the TUI yourself, open the Screen tab
#   nu scripts/soak.nu droidkraft 120         # the GUI
def main [name: string = "droidkraft", secs: int = 60, --every: int = 10] {
    let pids = (ps | where name == $name | get pid)
    if ($pids | is-empty) { error make --unspanned { msg: $"no running process named ($name)" } }
    let pid = ($pids | first)
    mut samples = []
    for t in (seq 0 $every $secs) {
        let mb = (ps | where pid == $pid | get 0.mem | into int | $in / 1_048_576 | math round --precision 1)
        let kids = (ps --long | where ppid == $pid | length)
        print $"($t)s  ($mb) MB  \(children: ($kids))"
        $samples = ($samples | append $mb)
        sleep ($every * 1sec)
    }
    let growth = (($samples | last) - ($samples | first) | math round --precision 1)
    print $"growth: ($growth) MB over ($secs)s"
}
