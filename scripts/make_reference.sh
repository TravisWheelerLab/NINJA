#!/usr/bin/env bash
# Regenerate the Java reference outputs under tests/reference.
#
# Requires the original NINJA jar (v1.2.2), available from
# https://wheelerlab.org/software/ninja/ (ninja.tgz contains Ninja.jar).
#
#   NINJA_JAR=/path/to/Ninja.jar scripts/make_reference.sh
set -euo pipefail

cd "$(dirname "$0")/.."
JAR="${NINJA_JAR:?set NINJA_JAR to the path of Ninja.jar}"
J="java -Xmx4G -jar $JAR"
F=tests/fixtures
R=tests/reference
mkdir -p "$R"

for f in PF08271_seed dna_200 protein_120 dna_700; do
  $J --in "$F/$f.fa" 2>/dev/null > "$R/$f.inmem.java.nwk"
  $J -m extmem --in "$F/$f.fa" 2>/dev/null > "$R/$f.extmem.java.nwk"
done
for f in PF08271_seed dna_200 protein_120; do
  $J --out_type d --in "$F/$f.fa" 2>/dev/null > "$R/$f.java.phylip"
done
$J --in_type d --in "$R/PF08271_seed.java.phylip" 2>/dev/null > "$R/PF08271_seed.fromphylip.java.nwk"

# dna_700's matrix is 4 MB, so it is not kept; build from a temporary copy.
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
$J --out_type d --in "$F/dna_700.fa" 2>/dev/null > "$tmp/dna_700.phylip"
$J --in_type d --in "$tmp/dna_700.phylip" 2>/dev/null > "$R/dna_700.fromphylip.java.nwk"
$J -m extmem --in_type d --in "$tmp/dna_700.phylip" 2>/dev/null > "$R/dna_700.fromphylip.extmem.java.nwk"

# The jar leaves its scratch directory next to the working directory.
rm -rf ninja_temp_*
ls -la "$R"
