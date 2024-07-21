#include "dt.h"

typedef union ValueValue {
    u64 nil;
    u64 number;
    // Object* ptr;
} ValueValue;

typedef struct Value {
    u64 type_or_pointer;
    ValueValue value;
} Value;

typedef struct VM {
    Value a;
    Value b;
    Value r;

    u8 const* const code;
    u64 const code_length;
    u8* const next_byte;

    // u8* offset_table;
    // u64 offset_table_length;

    Value stack[STACK_SIZE];
    u8* stack_pointer;

    // allocated_objects
} VM;

/*
inline VM VM__new(
    u8 const* const code,
    entry
) {

}*/