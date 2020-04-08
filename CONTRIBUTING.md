# Contributing

The following describe what you will need to know in order to
contribute to the NINJA project, whether that means code,
documentation, or otherwise.

## Development Tools

### Summary

  - CMake (minimum 3.10)
  - GCC (or non-Apple Clang)
  - [Google Test](https://github.com/google/googletest)
  - Dependencies are submodules: `git submodule init`

### Details

This project uses CMake, you'll need at least version 3.10.
Most of us use CLion as our IDE, but you can use whatever
you'd like.

We use GCC to build the project, although Clang should also
work. However, the version of Clang bundled with XCode does not
work at this time because it does not include OpenMP.

You can configure CLion to use GCC in the Preferences under
"Build, Execution, Deployment" > "CMake". Select GCC from the
"Toolchain" dropdown.

On the command line you can choose a compiler using the following
options:

```
-D CMAKE_C_COMPILER=/path/to/c_compiler
-D CMAKE_CXX_COMPILER=/path/to/c++_compiler
```

Some additional
[discussion](https://stackoverflow.com/questions/30399654/how-to-switch-between-gcc-and-clang-in-clion-from-within-cmakelists-txt-using-wi)
can be found on Stack Overflow.

#### Dependencies

We use Git submodules to manage dependencies because a) we
don't have many dependencies, and b) submodules are almost
trivial to use.

To fetch dependencies just use `git submodule init` at the
command line. You can also use whichever Git client you might
happen to prefer, as long as it handles submodules.
