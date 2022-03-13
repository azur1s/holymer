#pragma once
#include <iostream>
#include <fstream>
#include <string>

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

/*
 * @brief Read the value from the file and return it.
 * 
 * @param file_name The name of the file to read from.
 * @return std::string The value read from the file.
 */
std::string hazure_read_file(std::string filename) {
    std::ifstream file(filename);
    std::string content((std::istreambuf_iterator<char>(file)),
                        (std::istreambuf_iterator<char>()));
    return content;
}

/*
 * @brief Write string to file.
 * 
 * @param filename The file name to write to.
 * @param content The content to write.
 */
void hazure_write_file(std::string filename, std::string content) {
    std::ofstream file(filename);
    if (file.is_open()) {
        file << content;
        file.close();
    } else {
        std::cerr << "Unable to open " << filename << std::endl;
    }
}