# NINJA
Nearly Infinite Neighbor Joining Application

## About
TODO: Information from Travis... history, Java version, publication(s)

## Development
Clone the code. If you're on a Mac, you can run `make setup-mac` to install most
of the required tools through [Homebrew](http://brew.sh). This includes:

  - [pipenv](https://github.com/pypa/pipenv)
  - [pyenv](https://github.com/pyenv/pyenv)
  - [GCC 6](https://gcc.gnu.org)
  - [compiledb](https://github.com/nickdiego/compiledb)

Right now, the codebase uses [OpenMPI](https://www.open-mpi.org) which is not
included in the default C++ toolchain included with XCode on macOS. The easiest
way to get around this, at the moment, is to build with GCC. In the future, we
hope to eliminate the OpenMPI dependency so that the Mac version of Clang will
be able to build NINJA.

Contributions are welcomed. To do this, fork the repo, create a branch and, once
you're ready, create a pull request from your branch into the `master` branch in
the main repo.

## Authors

  - Travis Wheeler <travis.wheeler@umontana.edu>
  - Michel Wan Der Maas Soares
  - George Lesica <george.lesica@umontana.edu>
