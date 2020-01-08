## ----------------------------------------------------------------------------
## Makefile for NINJA
##
## Use `make help` for help on each target.
##
## Set the C++ compiler by setting CXX (for example, `make build
## CXX=/usr/local/bin/g++-9`). Note that the version of Clang that ships
## with MacOS does not include OpenMP and therefore won't work. We test
## with GCC 9, so we suggest using that. The C++ compiler you choose
## must support C++11 and OpenMP.
## ----------------------------------------------------------------------------

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
EXEC = ninja
DEBUGEXEC = ninja_debug

.PHONY: build
build: CXXFLAGS += -O3
build: $(OBJECTS) ## Build the production executable
	${CXX} $(CXXFLAGS) $(OBJECTS) -o $(EXEC) $(LNFLAGS)

.PHONY: build-debug
build-debug: CXXFLAGS += -O0 -g3 -pg
build-debug: $(OBJECTS) ## Build the debug executable
	${CXX} $(DBGFLAGS) $(CXXFLAGS) $(OBJECTS) -o $(DEBUGEXEC) $(LNFLAGS)

.PHONY: check
check: ## Run the main test suite (TODO)
	@echo "Running checks..."

.PHONY: check-format
check-format:
	@echo "Skip formatting for now"
#check-format: format ## Verify that the formatter has been run
#	git diff --exit-code

.PHONY: clean
clean: ## Remove build artifacts
	$(RM) count src/*.o src/*~
	$(RM) $(EXEC)
	$(RM) $(DEBUGEXEC)

.PHONY: clean-dist
clean-dist: ## Remove package artifacts
	$(RM) -r dist

.PHONY: format
format: ## Run the formatter (clang-format)
	clang-format -i -style=file src/*.cpp src/*.hpp

help: ## Show this help
	@sed -ne '/@sed/!s/## //p' $(MAKEFILE_LIST)

# +-----------+
# | Packaging |
# +-----------+

# Note: using -j will screw these up because they modify the working directory

WORK_DIR := $(shell pwd)

# The version string to use for packaging. If we are currently on
# a Git tag then that's what we'll use. Otherwise, this will be the
# most recent tag with the short version of the current hash appended.
VERSION := $(shell git describe --always --tags)

# Architecture that we're building on / for (e.g. X86_64).
ARCH := $(shell uname -m | tr -d '_')

# Name of the operating system or Linux distribution that we're
# building on / for (e.g. Ubuntu).
OS := $(shell scripts/get-os.sh)

# Operating system or distribution release version. This will vary
# depending on how the OS is versioned. For example, on Ubuntu we end
# up with something like "16.04", whereas on Fedora we just have a
# single integer. On MacOS we drop the patch version, so we get "10.15"
# instead of something like "10.15.3".
RELEASE := $(shell scripts/get-release.sh)

# Full version string used to name each package.
FULL_VERSION := 0.${VERSION}.${OS}.${RELEASE}.${ARCH}

KIND := "Unknown"
EXTENSION :=
DEPENDS :=
ifeq ($(OS), MacOSX)
    KIND := osxpkg
    EXTENSION := pkg
endif
ifeq ($(OS), Ubuntu)
    KIND := deb
    EXTENSION := deb
	DEPENDS += --depends libgomp1
endif
ifeq ($(OS), Fedora)
    KIND := rpm
    EXTENSION := rpm
	DEPENDS += --depends libgomp
endif
ifeq ($(OS), CentOS)
    KIND := rpm
    EXTENSION := rpm
	DEPENDS += --depends libgomp
endif
ifeq ($(KIND), Unknown)
    $(error "Unrecognized OS: ${OS}")
endif

.PHONY: package
package: clean build ## Create a binary package for the current platform
	mkdir -p dist
	fpm -s dir -t "${KIND}" -n ninja \
		${DEPENDS} \
		--epoch 0 \
		--description "NINJA package for ${OS} ${RELEASE}" \
		--force \
		--license "MIT" \
		--osxpkg-identifier-prefix "org.wheelerlab" \
		--package "dist/ninja-${FULL_VERSION}.${EXTENSION}" \
		--provides "ninja" \
		--url "https://github.com/TravisWheelerLab/NINJA" \
		--vendor "Wheeler Lab" \
		--verbose \
		--version "${FULL_VERSION}" \
		ninja=/usr/local/bin/

# +-----------------+
# | Cross-packaging |
# +-----------------+

# Note: using -j will screw these up because they modify the working directory

PKG_DIR := pkg
DOCKERFILES := $(notdir $(wildcard ${PKG_DIR}/Dockerfile*))
IMAGE_BASE_NAME := traviswheelerlab/ninja-build

.PHONY: package-linux
package-linux: ${DOCKERFILES:D%=package-linux-D%} ## Create all Linux binary packages (Docker)
package-linux-%: OS = $(shell echo $(word 2,$(subst -, , $*)) | tr [:upper:] [:lower:])
package-linux-%: RELEASE = $(shell echo $(word 3,$(subst -, ,$*)) | tr [:upper:] [:lower:])
package-linux-%: NAME = ${IMAGE_BASE_NAME}-${OS}-${RELEASE}
package-linux-%:
	@echo "Packaging for ${OS} ${RELEASE}"
	docker run \
		--mount src="${WORK_DIR}",target=/code,type=bind \
		--env VERSION="${VERSION}" \
		"${NAME}"

.PHONY: build-images
build-images: ${DOCKERFILES:D%=build-images-D%} ## Build Linux packaging images locally (Docker)
build-images-%: DOCKERFILE = $*
build-images-%: OS = $(shell echo $(word 2,$(subst -, , $*)) | tr [:upper:] [:lower:])
build-images-%: RELEASE = $(shell echo $(word 3,$(subst -, ,$*)) | tr [:upper:] [:lower:])
build-images-%: NAME = ${IMAGE_BASE_NAME}-${OS}-${RELEASE}
build-images-%:
	@echo "Building image for ${OS} ${RELEASE}"
	docker build -f "${PKG_DIR}/${DOCKERFILE}" -t "${NAME}" .
	@echo "Pushing ${NAME}"
	docker push "${NAME}"