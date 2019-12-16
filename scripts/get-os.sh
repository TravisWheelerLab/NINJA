#!/bin/sh

# Script to determine the name of the operating system we're 
# building on. Some Linux distros don't implement the LSB at
# all, so we may need other clauses if we decide to build on
# any of those.

# Fedora, Ubuntu, CentOS
OS=`lsb_release -i 2>/dev/null || echo ""`
if [ "$OS" != "" ]; then
    # Distributor ID:	Fedora --> Fedora
    # Distributor ID:	Ubuntu --> Ubuntu
    # Distributor ID:	CentOS --> CentOS
    echo "$OS" | tr -d [:blank:] | cut -d ':' -f 2
    exit 0
fi

# Mac
OS=`sw_vers -productName 2>/dev/null || echo ""`
if [ "$OS" != "" ]; then
    # Mac OS X --> MaxOSX
    echo "$OS" | tr -d [:blank:]
    exit 0
fi

echo "Unknown"
exit 1
