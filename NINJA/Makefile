# Makefile for NINJA
# Author: Michel Wan Der Maas
# 05/15/16


# only works if processor has support to SSE3, TODO: implement checking
# TODO: implement checking which operating system is being used
# TODO: implement flag for unit test compilation

#CXX = /usr/local/bin/g++-6 #must support C++11
CXX = /usr/bin/g++-7 #must support C++11

# sources in alphabetical order
SOURCES = ArgumentHandler.cpp
SOURCES += ArrayHeapExtMem.cpp
SOURCES += BinaryHeap_FourInts.cpp
SOURCES += BinaryHeap_IntKey_TwoInts.cpp
SOURCES += BinaryHeap_TwoInts.cpp
SOURCES += BinaryHeap.cpp
SOURCES += CandidateHeap.cpp
SOURCES += DistanceCalculatorAVX.cpp #switched from regular to AVX file here - NOT including other at the moment
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

#CXXFLAGS = -std=gnu++11 -Wall -ssse3 -fopenmp
CXXFLAGS = -std=gnu++11 -Wall -mavx2 -fopenmp
EXEC = Ninja
debugfile = Ninja_debug

debug: CXXFLAGS += -O0 -g3 -pg
all: CXXFLAGS += -O3

all: $(OBJECTS)
	${CXX} $(CXXFLAGS) $(OBJECTS) -o $(EXEC) $(LNFLAGS)

debug: $(OBJECTS)
	${CXX} $(DBGFLAGS) $(CXXFLAGS) $(OBJECTS) -o $(debugfile) $(LNFLAGS) 

clean:
	$(RM) count *.o *~
	$(RM) $(EXEC)
	$(RM) $(debugfile)
