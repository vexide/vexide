#include "api.h"

const uint32_t MAX_TIMEOUT = TIMEOUT_MAX;

int *errno_location() { return &errno; }