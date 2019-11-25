# Makefile for NINJA

# Set the C++ compiler. If you would like to use a particular compiler
# you can set it here or from the command line. The compiler must support
# C++11 and OpenMP (the version of Clang shipped by default with MacOS
# does not support the latter).
# CXX = /usr/bin/g++-7

SOURCES = ArgumentHandler.cpp
SOURCES += ArrayHeapExtMem.cpp
SOURCES += BinaryHeap.cpp
SOURCES += BinaryHeap_FourInts.cpp
SOURCES += BinaryHeap_IntKey_TwoInts.cpp
SOURCES += BinaryHeap_TwoInts.cpp
SOURCES += CandidateHeap.cpp
#SOURCES += DistanceCalculator.cpp
SOURCES += DistanceCalculatorAVX.cpp
SOURCES += DistanceReader.cpp
SOURCES += DistanceReaderExtMem.cpp
SOURCES += ExceptionHandler.cpp
SOURCES += Ninja.cpp
SOURCES += SequenceFileReader.cpp
SOURCES += Stack.cpp
SOURCES += TreeBuilder.cpp
SOURCES += TreeBuilderBinHeap.cpp
SOURCES += TreeBuilderExtMem.cpp
SOURCES += TreeBuilderManager.cpp
SOURCES += TreeNode.cpp
OBJECTS = $(SOURCES:.cpp=.o)

CXXFLAGS = -std=gnu++11 -Wall -mavx2 -msse3 -fopenmp
EXEC = Ninja
debugfile = Ninja_debug

.PHONY: all
all: CXXFLAGS += -O3
all: $(OBJECTS)
	${CXX} $(CXXFLAGS) $(OBJECTS) -o $(EXEC) $(LNFLAGS)

.PHONY: debug
debug: CXXFLAGS += -O0 -g3 -pg
debug: $(OBJECTS)
	${CXX} $(DBGFLAGS) $(CXXFLAGS) $(OBJECTS) -o $(debugfile) $(LNFLAGS) 

.PHONY: check
check:
	echo "Running checks..."

.PHONY: clean
clean:
	$(RM) count *.o *~
	$(RM) $(EXEC)
	$(RM) $(debugfile)
