#pragma once

enum INSTRUCTION {
    UNREACHABLE = 0x00,
    NO_OPERATION = 0x01,

    PUSH_A,
    PUSH_B,
    PUSH_R,
    POP,
    POP_INTO_A,
    POP_INTO_B,
    POP_INTO_R,
    TOP_INTO_A,
    TOP_INTO_B,
    TOP_INTO_R,
    SWAP,
    SWAP_A,
    SWAP_B,
    SWAP_R,
    DUP,
    CLEAR,

    #ifdef FEATURE_STANDARD_IO
    ARGS,
    STDOUT_WRITE,
    STDOUT_WRITE_LF,
    STDERR_WRITE,
    STDERR_WRITE_LF,
    STDIN_READ_LINE,
    STDIN_READ
    #endif

    #ifdef FEATURE_FILE_IO
    IO_IS_FILE,
    IO_IS_DIRECTORY,
    IO_CREATE_FILE,
    IO_FILE_READ,
    IO_FILE_WRITE,
    IO_SIZE,
    IO_MOVE,
    IO_COPY,

    #endif
}