# TODO: automatically check for SSE3
# TODO: implement checking which operating system is being used
# TODO: implement flag for unit test compilation

SOURCES := $(wildcard NINJA/*.cpp)
OBJECTS := $(SOURCES:.cpp=.o)

# The C++ compiler must support C++11
CXX := g++
CXXFLAGS := -std=gnu++11 -Wall
ifeq ($(shell uname -m),x86_64)
  CXXFLAGS += -mssse3
endif

# TODO: build to a dist directory or similar
# TODO: use separate release and debug build directories
EXEC := NINJA/Ninja

.PHONY: help
help:
	@echo benchmark - run very simple benchmarks on generated data
	@echo build     - create a production executable
	@echo check     - run the test suite
	@echo clean     - remove build and generated artifacts
	@echo compiledb - build the compilation database for CLion
	@echo debug     - create a debugging executable
	@echo docs      - update the auto-generated documentation
	@echo setup-mac - install development dependencies on a Mac

.PHONY: benchmark
benchmark: build fixtures/20-4.txt fixtures/10-2.txt fixtures/5-1.txt fixtures/1-1.txt
	time ./NINJA/Ninja --in fixtures/20-4.txt --out /dev/null
	time ./NINJA/Ninja --in fixtures/10-2.txt --out /dev/null
	time ./NINJA/Ninja --in fixtures/5-1.txt --out /dev/null
	time ./NINJA/Ninja --in fixtures/1-1.txt --out /dev/null

.PHONY: build
build: CXXFLAGS += -O3
build: $(OBJECTS)
	$(CXX) $(CXXFLAGS) $(OBJECTS) -o $(EXEC) $(LNFLAGS)

.PHONY: check
check: debug
	# Start with a simple functional test suite
	# that just runs a small number of sanity checks
	./NINJA/Ninja --in fixtures/PF08271_seed.txt

.PHONY: clean
clean:
	$(RM) $(OBJECTS)
	$(RM) $(EXEC)
	$(RM) fixtures/*-*.txt

## Build the compilation database used by CLion for code intelligence.
.PHONY: compiledb
compiledb:
	pipenv run compiledb -n make

.PHONY: debug
debug: CXXFLAGS += -O0 -g3
debug: $(OBJECTS)
	$(CXX) $(DBGFLAGS) $(CXXFLAGS) $(OBJECTS) -o $(EXEC) $(LNFLAGS)

.PHONY: docs
docs:
	doxygen Doxyfile

fixtures/20-4.txt:
	python fixtures/generate_benchmark.py 20000 4000 > fixtures/20-4.txt
fixtures/10-2.txt:
	python fixtures/generate_benchmark.py 10000 2000 > fixtures/10-2.txt
fixtures/5-1.txt:
	python fixtures/generate_benchmark.py 5000 1000 > fixtures/5-1.txt
fixtures/1-1.txt:
	python fixtures/generate_benchmark.py 1000 1000 > fixtures/1-1.txt

.PHONY: setup-mac
setup-mac:
	brew install doxygen
	brew install pipenv
	brew install python
	pipenv sync --dev
	@echo 'Use `pipenv shell` to start the virtual environment'
