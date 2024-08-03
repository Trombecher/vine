# Types And Values

Values on the VVM are 16 bytes. The first 8 bytes specify a _pointer_ to something. The last 8 bytes contain type specific values:

```
struct Value {
    type_pointer: usize,
    value_value: usize,
}
```

The type of a value can be a  or a composite type. It sets an interpretation of the _value-value_. Types of values are distinguished with their type pointers. If one value has the same type pointer as another value, then they have the same type.

The type pointer either points to nonsense (built-in type) or to a [_type information structure_ (TIS)](#tis) (composite type). Built-in types are differentiated from composite types by their type pointer:  
If the type pointer (value) matches a built-in type's pointer, then the pointer points to nonsense and the type of the value is determined.  
Otherwise the pointer points to a TIS and therefore the type is a composite type.

## Built-in Types

A value on the stack can have the following _type pointer_:

### `u64`

If a value has this type pointer, the value-value will be interpreted as an 8-byte unsigned integer (`u64`).

### `f64`

If a value has this type pointer, the value-value will be interpreted as an 8-byte floating point integer (`f64`).

### `widearray`

If a value has this type pointer, the value-value will be interpreted as an 8-byte pointer to this built-in structure:

```
struct WideArray {
    // Header
    length: usize,
    
    // Values
    entry_0: Value,
    entry_1: Value,
    entry_2: Value,
    ...
}
```

### `widevec`

If a value has this type pointer, the value-value will be interpreted as an 8-byte pointer to this built-in structure (where `Vec` is the Rust vector type):

```
struct WideVec {
    vec: *mut Vec<Value>,
}
```

### `smarray`

If a value has this type pointer, the value-value will be interpreted as an 8-byte pointer to this built-in structure:

```
struct SmallArray {
    // Header
    length: usize,
    type: usize,
    
    // (Small) values
    entry_0: usize,
    entry_1: usize,
    entry_2: usize,
    ...
}
```

### `smvec`

If a value has this type pointer, the value-value will be interpreted as an 8-byte pointer to this built-in structure:

```
struct SmallVec {
    type: usize,
    vec: *mut Vec<usize>
}
```

## Composite Types

Type pointers of composite types point to this structure:

### TIS

```
struct TIS {
    fields: usize,
}
```