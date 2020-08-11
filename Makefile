# TODO: automatically check for SSE3
# TODO: implement checking which operating system is being used
# TODO: implement flag for unit test compilation

SOURCES := $(wildcard NINJA/*.cpp)
OBJECTS := $(SOURCES:.cpp=.o)

# The C++ compiler must support C++11
CXX := g++
CXXFLAGS := -std=gnu++11 -Wall -mssse3

# TODO: build to a dist directory or similar
# TODO: use separate release and debug build directories
EXEC := NINJA/Ninja

.PHONY: all
all: CXXFLAGS += -O3
all: $(OBJECTS)
	$(CXX) $(CXXFLAGS) $(OBJECTS) -o $(EXEC) $(LNFLAGS)

.PHONY: check
check: debug
	# TODO: Run the test suite
	# Start with a simple functional test suite
	# that just runs a small number of sanity checks
	./NINJA/Ninja --in fixtures/PF08271_seed.txt

.PHONY: clean
clean:
	$(RM) $(OBJECTS)
	$(RM) $(EXEC)

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

.PHONY: setup-mac
setup-mac:
	brew install doxygen
	brew install pipenv
	brew install python
	pipenv sync --dev
	@echo 'Use `pipenv shell` to start the virtual environment'
