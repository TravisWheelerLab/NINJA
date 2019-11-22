# Makefile for NINJA

# Set the C++ compiler. If you would like to use a particular compiler
# you can set it here or from the command line. The compiler must support
# C++11 and OpenMP (the version of Clang shipped by default with MacOS
# does not support the latter).
# CXX = /usr/bin/g++-7

SOURCES = $(wildcard *.cpp)
OBJECTS = $(SOURCES:.cpp=.o)

CXXFLAGS = -std=gnu++11 -Wall -mavx2 -msse3 -fopenmp
EXEC = Ninja
debugfile = Ninja_debug

.PHONEY: all
all: CXXFLAGS += -O3
all: $(OBJECTS)
	${CXX} $(CXXFLAGS) $(OBJECTS) -o $(EXEC) $(LNFLAGS)

.PHONEY: debug
debug: CXXFLAGS += -O0 -g3 -pg
debug: $(OBJECTS)
	${CXX} $(DBGFLAGS) $(CXXFLAGS) $(OBJECTS) -o $(debugfile) $(LNFLAGS) 

.PHONEY: check
check:
	echo "Running checks..."

.PHONEY: clean
clean:
	$(RM) count *.o *~
	$(RM) $(EXEC)
	$(RM) $(debugfile)
