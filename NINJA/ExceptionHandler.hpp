#ifndef ExceptionHandler_HPP
#define ExceptionHandler_HPP

#include <cstdio>
#include <cstdlib>
#include <string>
#include <cstring>
#include <cassert>
#include <climits>
#include <algorithm>
#include <vector>
#include <utility>
#include <ctime>
#include <cerrno>

namespace Exception {
    void critical();

    void criticalErrno(const char *arg);
}

#endif
