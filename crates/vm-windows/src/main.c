#define STACK_SIZE 1024

#include <windows.h>

#include "dt.h"
#include "vm.h"



void write_file(LPCWSTR const file_name, u8 const* const content, u64 const content_length) {
    const HANDLE file_handle = CreateFileW(
        L"test.txt",
        GENERIC_WRITE,
        0,
        NULL,
        CREATE_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,
        NULL
    );

    WriteFile(
        file_handle,
        "Hello, World!",
        13,
        NULL,
        NULL
    );

    CloseHandle(file_handle);
}

int main() {
    
}