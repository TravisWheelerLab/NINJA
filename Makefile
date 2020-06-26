# TODO: automatically check for SSE3
# TODO: implement checking which operating system is being used
# TODO: implement flag for unit test compilation

SOURCES := $(wildcard NINJA/*.cpp)
OBJECTS = $(SOURCES:.cpp=.o)

# TODO: get newer GCC versions to work
# The C++ compiler must support C++11
CXX = g++-6
# TODO: eliminate the need for openmp so we can use Apple Clang
CXXFLAGS = -std=gnu++11 -Wall -mssse3

# TODO: build to a dist directory or similar
# TODO: use separate release and debug build directories
EXEC = NINJA/Ninja

all: CXXFLAGS += -O3
all: $(OBJECTS)
	${CXX} $(CXXFLAGS) $(OBJECTS) -o $(EXEC) $(LNFLAGS)

debug: CXXFLAGS += -O0 -g3 -pg
debug: $(OBJECTS)
	${CXX} $(DBGFLAGS) $(CXXFLAGS) $(OBJECTS) -o $(EXEC) $(LNFLAGS)

clean:
	$(RM) $(OBJECTS)
	$(RM) $(EXEC)

setup-mac:
	brew install pipenv
	brew install pyenv
	brew install gcc@6
	pyenv install -s 3.8.3
	pipenv sync --dev
	@echo 'Use `pipenv shell` to start the virtual environment'
