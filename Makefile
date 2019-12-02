# Makefile for NINJA

# Set the C++ compiler. If you would like to use a particular compiler
# you can set it here or from the command line. The compiler must support
# C++11 and OpenMP (the version of Clang shipped by default with MacOS
# does not support the latter).
# CXX = /usr/bin/g++-7

SOURCES = src/ArgumentHandler.cpp
SOURCES += src/ArrayHeapExtMem.cpp
SOURCES += src/BinaryHeap.cpp
SOURCES += src/BinaryHeap_FourInts.cpp
SOURCES += src/BinaryHeap_IntKey_TwoInts.cpp
SOURCES += src/BinaryHeap_TwoInts.cpp
SOURCES += src/CandidateHeap.cpp
#SOURCES += src/DistanceCalculator.cpp
SOURCES += src/DistanceCalculatorAVX.cpp
SOURCES += src/DistanceReader.cpp
SOURCES += src/DistanceReaderExtMem.cpp
SOURCES += src/ExceptionHandler.cpp
SOURCES += src/Ninja.cpp
SOURCES += src/SequenceFileReader.cpp
SOURCES += src/Stack.cpp
SOURCES += src/TreeBuilder.cpp
SOURCES += src/TreeBuilderBinHeap.cpp
SOURCES += src/TreeBuilderExtMem.cpp
SOURCES += src/TreeBuilderManager.cpp
SOURCES += src/TreeNode.cpp

OBJECTS = $(SOURCES:.cpp=.o)

CXXFLAGS = -std=gnu++11 -Wall -mavx2 -msse3 -fopenmp
EXEC = Ninja
debugfile = Ninja_debug

.PHONY: all
all: CXXFLAGS += -O3
all: $(OBJECTS)
	${CXX} $(CXXFLAGS) $(OBJECTS) -o $(EXEC) $(LNFLAGS)

.PHONY: check
check:
	echo "Running checks..."

.PHONY: clean
clean:
	$(RM) count src/*.o src/*~
	$(RM) $(EXEC)
	$(RM) $(debugfile)

.PHONY: debug
debug: CXXFLAGS += -O0 -g3 -pg
debug: $(OBJECTS)
	${CXX} $(DBGFLAGS) $(CXXFLAGS) $(OBJECTS) -o $(debugfile) $(LNFLAGS) 

.PHONY: format
format:
	clang-format -i -style=file src/*.cpp src/*.hpp

.PHONY: check-format
check-format: format
	git diff --exit-code

