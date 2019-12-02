#ifndef ExceptionHandler_HPP
#define ExceptionHandler_HPP

#include <algorithm>
#include <assert.h>
#include <errno.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <string>
#include <time.h>
#include <utility>
#include <vector>

namespace Exception {
void critical();
void criticalErrno(const char *arg);
} // namespace Exception

#endif
