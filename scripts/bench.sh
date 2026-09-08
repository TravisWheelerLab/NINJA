#!/usr/bin/env bash
# Time ninja on simulated alignments of increasing size.
#
#   scripts/bench.sh [sizes...]      (default: 2000 6000 20000)
#
# Prints wall time, CPU time and peak memory for the in-memory engine with
# all threads and with one thread, and for the external-memory engine with
# a 50 MB budget so that the matrix is paged to disk.
set -euo pipefail

cd "$(dirname "$0")/.."
cargo build --release --quiet
B=target/release/ninja
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

sizes=("$@")
[ ${#sizes[@]} -eq 0 ] && sizes=(2000 6000 20000)

fmt="  %-28s wall %es  cpu %Us  peak %M KB"
for n in "${sizes[@]}"; do
  python3 scripts/simulate_alignment.py "$n" 300 --seed 42 > "$work/$n.fa"
  echo "== $n taxa x 300 bp"
  /usr/bin/time -f "$fmt" -a -o /dev/stdout "$B" -q --in "$work/$n.fa" -o /dev/null \
    2>&1 | sed "s/^ /  inmem, all threads:       /" || true
  /usr/bin/time -f "$fmt" "$B" -q -T 1 --in "$work/$n.fa" -o /dev/null 2>&1 | tail -1 | sed 's/^  /  inmem, one thread   /'
  /usr/bin/time -f "$fmt" "$B" -q -m extmem --memory 0.05 -t "$work" --in "$work/$n.fa" -o /dev/null 2>&1 | tail -1 | sed 's/^  /  extmem, 50 MB budget/'
done
