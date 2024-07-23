export const RAW_ERRORS: Readonly<Record<number, RawError>> = {
    0: {
        source: "utf-8",
        description: "Illegal first byte.",
    },
    1: {
        source: "utf-8",
        description: "Expected continuation byte (UTF-8 byte 2).",
        context: "in a two byte UTF-8 code point encoding.",
    },
    3: {
        source: "utf-8",
        description: "Expected continuation byte (UTF-8 byte 2).",
        context: "in a three byte UTF-8 code point encoding.",
    },
    5: {
        source: "utf-8",
        description: "Expected continuation byte (UTF-8 byte 3).",
        context: "in a three byte UTF-8 code point encoding.",
    },
    7: {
        source: "utf-8",
        description: "Expected continuation byte (UTF-8 byte 2).",
        context: "in a four byte UTF-8 code point encoding.",
    },
    9: {
        source: "utf-8",
        description: "Expected continuation byte (UTF-8 byte 3).",
        context: "in a four byte UTF-8 code point encoding.",
    },
    11: {
        source: "utf-8",
        description: "Expected continuation byte (UTF-8 byte 4).",
        context: "in a four byte UTF-8 code point encoding.",
    },
    13: {
        source: "lexer",
        description: "Expected escape character.",
    },
    14: {
        source: "lexer",
        description: "Invalid string escape.",
        context: "after `\\\\` in a string escape.",
    },
    15: {
        source: "lexer",
        description: "Cannot use a keyword as a tag name.",
        context: "in a markup start tag.",
    },
    16: {
        source: "lexer",
        description: "Cannot use a keyword as a tag name.",
        context: "in a markup end tag.",
    },
    17: {
        source: "lexer",
        description: "Invalid (alphabetic) digit for decimal number literal. In decimal, you can only use numbers from zero to nine.",
        context: "while lexing a decimal number literal.",
    },
    18: {
        source: "lexer",
        description: "Expected a string escape character.",
        context: "in a string literal.",
    },
    19: {
        source: "lexer",
        description: "Expected something.",
        context: "in a string literal.",
    },
    20: {
        source: "lexer",
        description: "Non-ASCII characters in identifiers are currently not supported.",
        context: "while lexing an identifier.",
    },
    21: {
        source: "lexer",
        description: "Expected first hex digit.",
        context: "in the first digit of a string literal hex escape code.",
    },
    22: {
        source: "lexer",
        description: "Invalid hex digit (0-9A-F).",
        context: "in the first digit of a string literal hex escape code.",
    },
    23: {
        source: "lexer",
        description: "Valid hex digit, but out of range for an ascii character (`0..=0x7F`).",
        context: "in the first digit of a string literal hex escape code.",
    },
    24: {
        source: "lexer",
        description: "Expected second hex digit.",
        context: "in the second digit of a string literal hex escape code.",
    },
    25: {
        source: "lexer",
        description: "Invalid hex digit (0-9A-F).",
        context: "in the second digit of a string literal hex escape code.",
    },
    26: {
        source: "lexer",
        description: "Expected `>`.",
        context: "in a markup end tag.",
    },
    27: {
        source: "lexer",
        description: "Vine does not have the Rust and C++ like :: path separator; just use a dot.",
    },
    28: {
        source: "lexer",
        description: "Illegal character.",
        context: "while trying to construct a new token.",
    },
    29: {
        source: "lexer",
        description: "Expected a number, found an alphabetic character.",
        context: "while lexing the tail of a floating point decimal number.",
    },
    30: {
        source: "lexer",
        description: "Expected a character.",
        context: "while lexing a character literal.",
    },
    31: {
        source: "lexer",
        description: "Expected a single quote to close the character literal.",
        context: "while lexing a character literal.",
    },
    32: {
        source: "lexer",
        description: "Expected a single quote to close the character literal.",
        context: "while lexing a character literal.",
    },
    33: {
        source: "lexer",
        description: "Expected `>`.",
        context: "in a self-closing markup tag.",
    },
    34: {
        source: "lexer",
        description: "Expected `>`, `/` or an identifier.",
        context: "while collecting attributes in a markup start tag.",
    },
    35: {
        source: "lexer",
        description: "Expected `=`.",
        context: "while lexing an attribute in a markup start tag.",
    },
    36: {
        source: "lexer",
        description: "Expected `\"` or `{`.",
        context: "while lexing a value in a markup start tag.",
    },
    37: {
        source: "lexer",
        description: "Cannot terminate without closing the markup element.",
        context: "in a markup element (while lexing children).",
    },
    38: {
        source: "lexer",
        description: "Expected `/` or a start tag identifier.",
        context: "in a markup element (while lexing children).",
    },
    39: {
        source: "parser",
        description: "Expected `mod`, `fn`, `use`, `let` or `struct`.",
        context: "while parsing a statement in a module.",
    },
    40: {
        source: "parser",
        description: "Expected identifier (type parameter) or `>`.",
        context: "in type parameters.",
    },
    41: {
        source: "parser",
        description: "Expected `mod`, `fn`, `use`, `let` or `struct`.",
        context: "while parsing a mandatory statement because of previous annotations.",
    },
    42: {
        source: "parser",
        description: "Expected an identifier.",
        context: "while parsing an annotation.",
    },
    43: {
        source: "parser",
        description: "Expected identifier (child) or `}`.",
        context: "in `use`-statement children.",
    },
    44: {
        source: "parser",
        description: "Expected `,` or `}`, but got `;`.",
        context: "after an identifier in a `use`-statement.",
    },
    45: {
        source: "parser",
        description: "Expected identifier, `{` or `*`.",
        context: "in a `use`-statement path.",
    },
    46: {
        source: "parser",
        description: "Expected `;`, `,`, `}` or '.'.",
        context: "in a `use`-statement.",
    },
    47: {
        source: "parser",
        description: "Expected an identifier, `*` or `{`.",
        context: "in a root `use`-statement.",
    },
    48: {
        source: "parser",
        description: "Expected an identifier or `.`.",
        context: "after `use`.",
    },
    49: {
        source: "parser",
        description: "Expected `;` or `}`.",
        context: "at the end of a block.",
    },
    50: {
        source: "parser",
        description: "Expected identifier.",
        context: "in a declaration.",
    },
    51: {
        source: "parser",
        description: "Expected identifier.",
        context: "in a `struct`-statement.",
    },
    52: {
        source: "parser",
        description: "Expected `(`.",
        context: "in a `struct`-statement. If you were trying to declare type parameters, move them right after the `struct` keyword.",
    },
    53: {
        source: "parser",
        description: "Expected identifier.",
        context: "in a struct field declaration.",
    },
    54: {
        source: "parser",
        description: "Expected `,` or `)`.",
        context: "After a struct field declaration.",
    },
    55: {
        source: "parser",
        description: "Tag names do not match.",
        context: "while parsing a markup end tag.",
    },
    56: {
        source: "parser",
        description: "Expected `,` or `)`.",
        context: "in function parameters, after `this`.",
    },
    57: {
        source: "parser",
        description: "Expected an identifier.",
        context: "in a function parameter.",
    },
    58: {
        source: "parser",
        description: "Expected an identifier.",
        context: "in a type path.",
    },
    59: {
        source: "parser",
        description: "Expected an identifier, `!` or `fn`.",
        context: "while parsing a type.",
    },
    60: {
        source: "parser",
        description: "Expected `(`.",
        context: "while parsing a function expression.",
    },
    61: {
        source: "parser",
        description: "Expected `{`.",
        context: "while parsing the body of a function expression. If you would like this function expression to be bodyless, remove the return type."
    },
    62: {
        source: "parser",
        description: "Expected `{`.",
        context: "while parsing the body of an if-expression. Help: Vine uses Rust-like syntax for if-expressions, rather than C- or Java-like syntax."
    },
    63: {
        source: "parser",
        description: "Expected `if` or `{`.",
        context: "in an else-chain."
    },
    64: {
        source: "parser",
        description: "Expected `{`.",
        context: "while parsing the body of an else-if-chain."
    },
    65: {
        source: "parser",
        description: "Expected `{`.",
        context: "while a while-expression."
    },
    66: {
        source: "parser",
        description: "Unexpected token.",
        context: "while parsing the beginning of an expression."
    },
    67: {
        source: "parser",
        description: "Invalid assignment target. It must be an an identifier or an access expression.",
        context: "while parsing an assignment expression"
    },
    68: {
        source: "parser",
        description: "Expected an identifier.",
        context: "in an access expression."
    },
    69: {
        source: "parser",
        description: "Expected `)` or `,`.",
        context: "in a call expression."
    },
    70: {
        source: "parser",
        description: "Unexpected token.",
        context: "while parsing an expression extension."
    },
    71: {
        source: "parser",
        description: "Expected an identifier.",
        context: "in a module declaration."
    },
    72: {
        source: "parser",
        description: "Missing `}`.",
        context: "after the module content.",
    },
    73: {
        source: "parser",
        description: "Expected `{` or `;`.",
        context: "in a module declaration",
    },
    74: {
        source: "parser",
        description: "Expected an identifier.",
        context: "in a function statement, after the `fn` keyword and type parameters.",
    },
    75: {
        source: "parser",
        description: "Expected `{` or `->`.",
        context: "in a function statement. Help: if you want to use a bodyless function, split the function statement into declaration and expression."
    },
    76: {
        source: "parser",
        description: "Expected `(`.",
        context: "in a function statement (parameters)."
    },
    77: {
        source: "parser",
        description: "Expected `{`.",
        context: "in a function statement. After a return type, there must always be a braced function body."
    }
};

export type RawError = {
    source: keyof typeof SOURCES,
    description: string,
    context?: string,
};

export const SOURCES = {
    "utf-8": "UTF8",
    "lexer": "Lexer",
    "parser": "Parser",
} as const;