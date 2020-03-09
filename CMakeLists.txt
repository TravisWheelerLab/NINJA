cmake_minimum_required(VERSION 3.10)
project(NINJA)

set(CMAKE_CXX_STANDARD 14)
set(CMAKE_C_FLAGS "-std=gnu++11 -Wall -mssse3 -fopenmp")
set(CMAKE_C_FLAGS_DEBUG "-std=gnu++11 -Wall -mssse3 -fopenmp  -O0 -g3 -pg")
set(CMAKE_C_FLAGS_RELEASE "-std=gnu++11 -Wall -mssse3 -fopenmp -O3")
set(CMAKE_CXX_FLAGS "-std=gnu++11 -Wall -mssse3 -fopenmp -O3")
set(CMAKE_CXX_FLAGS_DEBUG "-std=gnu++11 -Wall -mssse3 -fopenmp  -O0 -g3 -pg")
set(CMAKE_CXX_FLAGS_RELEASE "-std=gnu++11 -Wall -mssse3 -fopenmp -O3")

include_directories(src)

add_subdirectory(src)
add_subdirectory(test)
add_subdirectory(lib/googletest)


