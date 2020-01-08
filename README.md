# NINJA
Nearly Infinite Neighbor Joining Application

## Building

There are a number of make targets for different situations. Run
`make help` to see a list along with descriptions of the developer-facing
targets.

### Linux

Nothing special should be required to build on a reasonably recent
distribution.

From the `NINJA` subdirectory:

```
make
```

### Mac

Macs ship with a version of Clang that, as of Catalina (10.15) does not
appear to support OpenMP. Consequently, the easiest way to build NINJA is to
install GCC using [Homebrew](https://brew.sh). The default GCC package in
Homebrew as of this writing is version 9. If you would like to use a
different version simply update the `g++-9` command below.

From the `NINJA` subdirectory:

```
brew install gcc
make CXX=/usr/local/bin/g++-9
```

### Docker

It is possible to use Docker to build a Linux binary, even if your machine
doesn't have the exact requirements. This is also helpful for
cross-compilation on a Mac (for example). To build with Docker, run the
command below from the repository root:

```
docker run --mount src="$(pwd)",target=/code,type=bind traviswheelerlab/ninja-build:latest
```

This will download the latest version of the build image. See
<https://hub.docker.com/repository/docker/wheelerlabum/ninja-build>
for other versions. It will build the code in your working copy of the
repository and produce an executable called `Ninja`.

#### Other Images

There are a number of other Docker images defined in the `pkg` directory for various
Linux distributions. To build the images locally, run `make build-images`. To build
NINJA using one of these images (regardless of whether you've built it locally), run
the following:

```
docker run --mount src="$(pwd)",target=/code,type=bind traviswheelerlab/ninja-build-{OS}-{RELEASE}
```

Substitute `OS` and `RELEASE` for any distribution / version pair that has a Dockerfile
defined in `pkg/`. For example, "Fedora" and "31" will result in a build on Fedora 31.
These containers will run a build in the local directory and also create a distribution
package (like an RPM), depending on the OS chosen.

To generate all Linux packages run `make package-linux`. To generate a Mac installer
package, run `make package` on a Mac.

Packages will be saved in the `dist` directory (which will be created if necessary).

## Contributing

We use a "git flow" workflow. We have two active branches:
 * **master** is to become the stable NINJA release branch. 
 * **develop** is the NINJA development branch


To clone your own copy of NINJA source code repository for the first time:

```bash
   $ git clone https://github.com/TravisWheelerLab/NINJA
   $ cd NINJA
   $ git checkout develop
```

To contribute to NINJA development, you want to be on the
**develop** branch, which is where we are currently integrating
feature branches. For more information, see the
[NINJA wiki](https://github.com/TravisWheelerLab/NINJA/wiki).
