#pragma once
#include <iostream>

template<typename T>
/**
 * @brief Read the value from stdin and return it.
 */
T hazure_read() {
    T x;
    std::cin >> x;
    return x;
}

template<typename T>
/**
 * @brief Prints the value of the variable to the stdout.
 * 
 * @param value The value to print.
 */
void hazure_write(T x) {
    std::cout << x;
}