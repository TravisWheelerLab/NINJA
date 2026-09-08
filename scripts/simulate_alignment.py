#!/usr/bin/env python3
"""Simulate an alignment by evolving sequences down a random tree.

Usage: simulate_alignment.py N LENGTH [--protein] [--seed S] [--gap-rate R]
                             [--tree-out FILE] > alignment.fa

A random rooted binary tree over N taxa is built by repeatedly joining two
random subtrees, with branch lengths drawn from an exponential distribution.
Sequences evolve along it under a Jukes-Cantor-style model (each site is
replaced by a uniformly random residue with probability 1 - exp(-4/3 t) for
DNA, 1 - exp(-20/19 t) for protein). Gaps are inserted at random columns per
sequence at --gap-rate. The true tree is written in Newick to --tree-out.
"""
import argparse
import math
import random
import sys

DNA = "ACGT"
AA = "ARNDCQEGHILKMFPSTWYV"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("n", type=int)
    ap.add_argument("length", type=int)
    ap.add_argument("--protein", action="store_true")
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--gap-rate", type=float, default=0.02)
    ap.add_argument("--mean-branch", type=float, default=0.05)
    ap.add_argument("--tree-out")
    a = ap.parse_args()
    rng = random.Random(a.seed)
    alphabet = AA if a.protein else DNA
    k = len(alphabet)
    p_same = 1.0 / k

    # Build a random tree: nodes are (name, children, branch_length).
    nodes = [(f"seq{i}", [], rng.expovariate(1 / a.mean_branch)) for i in range(a.n)]
    while len(nodes) > 1:
        i, j = rng.sample(range(len(nodes)), 2)
        a_, b_ = nodes[i], nodes[j]
        for idx in sorted((i, j), reverse=True):
            nodes.pop(idx)
        nodes.append(("", [a_, b_], rng.expovariate(1 / a.mean_branch)))
    root = nodes[0]

    def newick(n):
        name, kids, bl = n
        if kids:
            return "(" + ",".join(newick(c) for c in kids) + ")" + f":{bl:.5f}"
        return f"{name}:{bl:.5f}"

    if a.tree_out:
        with open(a.tree_out, "w") as f:
            f.write("(" + ",".join(newick(c) for c in root[1]) + ");\n")

    seqs = {}
    stack = [(root, [rng.choice(alphabet) for _ in range(a.length)])]
    while stack:
        (name, kids, bl), seq = stack.pop()
        if not kids:
            seqs[name] = seq
            continue
        for child in kids:
            t = child[2]
            p_change = (1 - p_same) * (1 - math.exp(-t / (1 - p_same)))
            new = [c if rng.random() >= p_change else rng.choice(alphabet) for c in seq]
            stack.append((child, new))

    out = sys.stdout
    for i in range(a.n):
        name = f"seq{i}"
        s = seqs[name]
        s = ["-" if rng.random() < a.gap_rate else c for c in s]
        out.write(f">{name}\n")
        for off in range(0, a.length, 60):
            out.write("".join(s[off:off + 60]) + "\n")


if __name__ == "__main__":
    main()
