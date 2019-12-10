#!/bin/sh

# Script to determine the name of the operating system release
# we're building on. Some Linux distros don't implement the LSB
# at all, so we may need other clauses if we decide to build on
# any of those.

# Fedora, Ubuntu, CentOS
RELEASE=`lsb_release -r 2>/dev/null || echo ""`
if [ "$RELEASE" != "" ]; then
    full_version=`echo "$RELEASE" | tr -d [:blank:] | cut -d ':' -f 2`
    part_n=`echo "$full_version" | tr '.' '\n' | wc -l`

    # Release:  7.7.1908 --> 7
    if [ "$part_n" = '3' ]; then
        echo "$full_version" | cut -d '.' -f 1
        exit 0
    fi

    # Release:	31 --> 31
    # Release:	16.04 --> 16.04
    echo "$full_version"
    exit 0
fi

# Mac
RELEASE=`sw_vers -productVersion 2>/dev/null || echo ""`
if [ "$RELEASE" != "" ]; then
    # 10.15.1 --> 10.15
    echo "$RELEASE" | cut -d '.' -f 1-2
    exit 0
fi

echo "Unknown"
exit 1
