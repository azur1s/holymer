#pragma once
#include <ctime>

/*
 * @brief Get time in seconds since the Epoch.
 */
int hazure_get_time() {
    return std::time(0);
}