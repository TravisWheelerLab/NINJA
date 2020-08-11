#include "ExceptionHandler.hpp"

void Exception::critical() {
    fprintf(stderr, "Critical Error, aborting.\n");
    abort();
}

void Exception::criticalErrno(const char *arg) {
    if (arg != nullptr) {
        perror(arg);
    } else {
        strerror(errno);
    }
    fprintf(stderr, "Critical Error, aborting.\n");
    abort();
}
