# Makefile for NINJA
# Author: Michel Wan Der Maas
# 05/15/16


# only works if processor has support to SSE3, TODO: implement checking
# TODO: implement checking which operating system is being used
# TODO: implement flag for unit test compilation

CXX = g++

# sources in alphabetical order
SOURCES = ArgumentHandler.cpp
SOURCES += ArrayHeapExtMem.cpp
SOURCES += BinaryHeap_FourInts.cpp
SOURCES += BinaryHeap_IntKey_TwoInts.cpp
SOURCES += BinaryHeap_TwoInts.cpp
SOURCES += BinaryHeap.cpp
SOURCES += CandidateHeap.cpp
SOURCES += DistanceCalculator.cpp
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

CXXFLAGS = -std=gnu++11 -O3 -Wall -mssse3 -fopenmp
DBGFLAGS = -O0 -g3 -pg -std=gnu++11 -Wall -mssse3 -fopenmp
EXEC = Ninja
debugfile = Ninja_debug

all: $(OBJECTS)
	g++ $(CXXFLAGS) $(OBJECTS) -o $(EXEC) $(LNFLAGS)

debug: $(OBJECTS)
	g++ $(DBXFLAGS) $(OBJECTS) -o $(EXEC) $(LNFLAGS) 

clean:
	$(RM) count *.o *~
