#include "api.h"

const uint32_t timeout_max = TIMEOUT_MAX;

int* errno_location() {
    return &errno;
}