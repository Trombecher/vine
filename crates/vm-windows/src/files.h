#pragma once

typedef struct DirectoryIterator {
    HANDLE current_file;
} DirectoryIterator;

void next(DirectoryIterator* const iter) {
    
}

void read_directory(LPCWSTR const directory_name) {
    WIN32_FIND_DATA const find_data;
    
    HANDLE current_file = FindFirstFileW(directory_name, &find_data);
    if(current_file == INVALID_HANDLE_VALUE) return;

    do {
        
    } while(FindNextFileW(current_file, &data));
}