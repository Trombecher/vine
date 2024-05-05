# Functions

## Attributes

Functions can have attributes, altering their behavior at runtime or compile-time. Parenthesised attributes should be inferred by the compiler and applied platform dependently.

- Pure: The function (a) always returns the same value for a set of parameters and (b) has no side effects.
- Asynchronous
- (Constant): The function only uses constant expressions and invocations of constant functions.
- (Inline): The function is inlined