//! This file was automatically generated.

#[derive(PartialEq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum Error {
    /// Illegal first byte.
    E0000 = 0,

    /// Expected continuation byte (UTF-8 byte 2).
    ///
    /// Context: The error occurred in a two byte UTF-8 code point encoding.
    E0001 = 1,

    /// Expected continuation byte (UTF-8 byte 2).
    ///
    /// Context: The error occurred in a three byte UTF-8 code point encoding.
    E0003 = 3,

    /// Expected continuation byte (UTF-8 byte 3).
    ///
    /// Context: The error occurred in a three byte UTF-8 code point encoding.
    E0005 = 5,

    /// Expected continuation byte (UTF-8 byte 2).
    ///
    /// Context: The error occurred in a four byte UTF-8 code point encoding.
    E0007 = 7,

    /// Expected continuation byte (UTF-8 byte 3).
    ///
    /// Context: The error occurred in a four byte UTF-8 code point encoding.
    E0009 = 9,

    /// Expected continuation byte (UTF-8 byte 4).
    ///
    /// Context: The error occurred in a four byte UTF-8 code point encoding.
    E0011 = 11,

    /// Expected escape character.
    E0013 = 13,

    /// Invalid string escape.
    ///
    /// Context: The error occurred after `\\` in a string escape.
    E0014 = 14,

    /// Cannot use a keyword as a tag name.
    ///
    /// Context: The error occurred in a markup start tag.
    E0015 = 15,

    /// Cannot use a keyword as a tag name.
    ///
    /// Context: The error occurred in a markup end tag.
    E0016 = 16,

    /// Invalid (alphabetic) digit for decimal number literal. In decimal, you can only use numbers from zero to nine.
    ///
    /// Context: The error occurred while lexing a decimal number literal.
    E0017 = 17,

    /// Expected a string escape character.
    ///
    /// Context: The error occurred in a string literal.
    E0018 = 18,

    /// Expected something.
    ///
    /// Context: The error occurred in a string literal.
    E0019 = 19,

    /// Non-ASCII characters in identifiers are currently not supported.
    ///
    /// Context: The error occurred while lexing an identifier.
    E0020 = 20,

    /// Expected first hex digit.
    ///
    /// Context: The error occurred in the first digit of a string literal hex escape code.
    E0021 = 21,

    /// Invalid hex digit (0-9A-F).
    ///
    /// Context: The error occurred in the first digit of a string literal hex escape code.
    E0022 = 22,

    /// Valid hex digit, but out of range for an ascii character (`0..=0x7F`).
    ///
    /// Context: The error occurred in the first digit of a string literal hex escape code.
    E0023 = 23,

    /// Expected second hex digit.
    ///
    /// Context: The error occurred in the second digit of a string literal hex escape code.
    E0024 = 24,

    /// Invalid hex digit (0-9A-F).
    ///
    /// Context: The error occurred in the second digit of a string literal hex escape code.
    E0025 = 25,

    /// Expected `>`.
    ///
    /// Context: The error occurred in a markup end tag.
    E0026 = 26,

    /// Vine does not have the Rust and C++ like :: path separator; just use a dot.
    E0027 = 27,

    /// Illegal character.
    ///
    /// Context: The error occurred while trying to construct a new token.
    E0028 = 28,

    /// Expected a number, found an alphabetic character.
    ///
    /// Context: The error occurred while lexing the tail of a floating point decimal number.
    E0029 = 29,

    /// Expected a character.
    ///
    /// Context: The error occurred while lexing a character literal.
    E0030 = 30,

    /// Expected a single quote to close the character literal.
    ///
    /// Context: The error occurred while lexing a character literal.
    E0031 = 31,

    /// Expected a single quote to close the character literal.
    ///
    /// Context: The error occurred while lexing a character literal.
    E0032 = 32,

    /// Expected `>`.
    ///
    /// Context: The error occurred in a self-closing markup tag.
    E0033 = 33,

    /// Expected `>`, `/` or an identifier.
    ///
    /// Context: The error occurred while collecting attributes in a markup start tag.
    E0034 = 34,

    /// Expected `=`.
    ///
    /// Context: The error occurred while lexing an attribute in a markup start tag.
    E0035 = 35,

    /// Expected `"` or `{`.
    ///
    /// Context: The error occurred while lexing a value in a markup start tag.
    E0036 = 36,

    /// Cannot terminate without closing the markup element.
    ///
    /// Context: The error occurred in a markup element (while lexing children).
    E0037 = 37,

    /// Expected `/` or a start tag identifier.
    ///
    /// Context: The error occurred in a markup element (while lexing children).
    E0038 = 38,

    /// Expected `mod`, `fn`, `use`, `let` or `struct`.
    ///
    /// Context: The error occurred while parsing a statement in a module.
    E0039 = 39,

    /// Expected identifier (type parameter) or `>`.
    ///
    /// Context: The error occurred in type parameters.
    E0040 = 40,

    /// Expected `mod`, `fn`, `use`, `let` or `struct`.
    ///
    /// Context: The error occurred while parsing a mandatory statement because of previous annotations.
    E0041 = 41,

    /// Expected an identifier.
    ///
    /// Context: The error occurred while parsing an annotation.
    E0042 = 42,

    /// Expected identifier (child) or `}`.
    ///
    /// Context: The error occurred in `use`-statement children.
    E0043 = 43,

    /// Expected `,` or `}`, but got `;`.
    ///
    /// Context: The error occurred after an identifier in a `use`-statement.
    E0044 = 44,

    /// Expected identifier, `{` or `*`.
    ///
    /// Context: The error occurred in a `use`-statement path.
    E0045 = 45,

    /// Expected `;`, `,`, `}` or '.'.
    ///
    /// Context: The error occurred in a `use`-statement.
    E0046 = 46,

    /// Expected an identifier, `*` or `{`.
    ///
    /// Context: The error occurred in a root `use`-statement.
    E0047 = 47,

    /// Expected an identifier or `.`.
    ///
    /// Context: The error occurred after `use`.
    E0048 = 48,

    /// Expected `;` or `}`.
    ///
    /// Context: The error occurred at the end of a block.
    E0049 = 49,

    /// Expected identifier.
    ///
    /// Context: The error occurred in a declaration.
    E0050 = 50,

    /// Expected identifier.
    ///
    /// Context: The error occurred in a `struct`-statement.
    E0051 = 51,

    /// Expected `(`.
    ///
    /// Context: The error occurred in a `struct`-statement. If you were trying to declare type parameters, move them right after the `struct` keyword.
    E0052 = 52,

    /// Expected identifier.
    ///
    /// Context: The error occurred in a struct field declaration.
    E0053 = 53,

    /// Expected `,` or `)`.
    ///
    /// Context: The error occurred After a struct field declaration.
    E0054 = 54,

    /// Tag names do not match.
    ///
    /// Context: The error occurred while parsing a markup end tag.
    E0055 = 55,

    /// Expected `,` or `)`.
    ///
    /// Context: The error occurred in function parameters, after `this`.
    E0056 = 56,

    /// Expected an identifier.
    ///
    /// Context: The error occurred in a function parameter.
    E0057 = 57,

    /// Expected an identifier.
    ///
    /// Context: The error occurred in a type path.
    E0058 = 58,

    /// Expected an identifier, `!` or `fn`.
    ///
    /// Context: The error occurred while parsing a type.
    E0059 = 59,

    /// Expected `(`.
    ///
    /// Context: The error occurred while parsing a function expression.
    E0060 = 60,

    /// Expected `{`.
    ///
    /// Context: The error occurred while parsing the body of a function expression. If you would like this function expression to be bodyless, remove the return type.
    E0061 = 61,

    /// Expected `{`.
    ///
    /// Context: The error occurred while parsing the body of an if-expression. Help: Vine uses Rust-like syntax for if-expressions, rather than C- or Java-like syntax.
    E0062 = 62,

    /// Expected `if` or `{`.
    ///
    /// Context: The error occurred in an else-chain.
    E0063 = 63,

    /// Expected `{`.
    ///
    /// Context: The error occurred while parsing the body of an else-if-chain.
    E0064 = 64,

    /// Expected `{`.
    ///
    /// Context: The error occurred while a while-expression.
    E0065 = 65,

    /// Unexpected token.
    ///
    /// Context: The error occurred while parsing the beginning of an expression.
    E0066 = 66,

    /// Invalid assignment target. It must be an an identifier or an access expression.
    ///
    /// Context: The error occurred while parsing an assignment expression
    E0067 = 67,

    /// Expected an identifier.
    ///
    /// Context: The error occurred in an access expression.
    E0068 = 68,

    /// Expected `)` or `,`.
    ///
    /// Context: The error occurred in a call expression.
    E0069 = 69,

    /// Unexpected token.
    ///
    /// Context: The error occurred while parsing an expression extension.
    E0070 = 70,

    /// Expected an identifier.
    ///
    /// Context: The error occurred in a module declaration.
    E0071 = 71,

    /// Missing `}`.
    ///
    /// Context: The error occurred after the module content.
    E0072 = 72,

    /// Expected `{` or `;`.
    ///
    /// Context: The error occurred in a module declaration
    E0073 = 73,

    /// Expected an identifier.
    ///
    /// Context: The error occurred in a function statement, after the `fn` keyword and type parameters.
    E0074 = 74,

    /// Expected `{` or `->`.
    ///
    /// Context: The error occurred in a function statement. Help: if you want to use a bodyless function, split the function statement into declaration and expression.
    E0075 = 75,

    /// Expected `(`.
    ///
    /// Context: The error occurred in a function statement (parameters).
    E0076 = 76,

    /// Expected `{`.
    ///
    /// Context: The error occurred in a function statement. After a return type, there must always be a braced function body.
    E0077 = 77,

    /// Unexpected token
    E0078 = 78,
}

#[derive(PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Source {
    UTF8,
    Lexer,
    Parser,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Source::UTF8 => "utf-8",
            Source::Lexer => "lexer",
            Source::Parser => "parser",
        }
    }
}

impl Error {
    pub fn as_str(self) -> &'static str {
        match self {
            Error::E0000 => "Illegal first byte.",
            Error::E0001 => "Expected continuation byte (UTF-8 byte 2).",
            Error::E0003 => "Expected continuation byte (UTF-8 byte 2).",
            Error::E0005 => "Expected continuation byte (UTF-8 byte 3).",
            Error::E0007 => "Expected continuation byte (UTF-8 byte 2).",
            Error::E0009 => "Expected continuation byte (UTF-8 byte 3).",
            Error::E0011 => "Expected continuation byte (UTF-8 byte 4).",
            Error::E0013 => "Expected escape character.",
            Error::E0014 => "Invalid string escape.",
            Error::E0015 => "Cannot use a keyword as a tag name.",
            Error::E0016 => "Cannot use a keyword as a tag name.",
            Error::E0017 => "Invalid (alphabetic) digit for decimal number literal. In decimal, you can only use numbers from zero to nine.",
            Error::E0018 => "Expected a string escape character.",
            Error::E0019 => "Expected something.",
            Error::E0020 => "Non-ASCII characters in identifiers are currently not supported.",
            Error::E0021 => "Expected first hex digit.",
            Error::E0022 => "Invalid hex digit (0-9A-F).",
            Error::E0023 => "Valid hex digit, but out of range for an ascii character (`0..=0x7F`).",
            Error::E0024 => "Expected second hex digit.",
            Error::E0025 => "Invalid hex digit (0-9A-F).",
            Error::E0026 => "Expected `>`.",
            Error::E0027 => "Vine does not have the Rust and C++ like :: path separator; just use a dot.",
            Error::E0028 => "Illegal character.",
            Error::E0029 => "Expected a number, found an alphabetic character.",
            Error::E0030 => "Expected a character.",
            Error::E0031 => "Expected a single quote to close the character literal.",
            Error::E0032 => "Expected a single quote to close the character literal.",
            Error::E0033 => "Expected `>`.",
            Error::E0034 => "Expected `>`, `/` or an identifier.",
            Error::E0035 => "Expected `=`.",
            Error::E0036 => "Expected `\"` or `{`.",
            Error::E0037 => "Cannot terminate without closing the markup element.",
            Error::E0038 => "Expected `/` or a start tag identifier.",
            Error::E0039 => "Expected `mod`, `fn`, `use`, `let` or `struct`.",
            Error::E0040 => "Expected identifier (type parameter) or `>`.",
            Error::E0041 => "Expected `mod`, `fn`, `use`, `let` or `struct`.",
            Error::E0042 => "Expected an identifier.",
            Error::E0043 => "Expected identifier (child) or `}`.",
            Error::E0044 => "Expected `,` or `}`, but got `;`.",
            Error::E0045 => "Expected identifier, `{` or `*`.",
            Error::E0046 => "Expected `;`, `,`, `}` or '.'.",
            Error::E0047 => "Expected an identifier, `*` or `{`.",
            Error::E0048 => "Expected an identifier or `.`.",
            Error::E0049 => "Expected `;` or `}`.",
            Error::E0050 => "Expected identifier.",
            Error::E0051 => "Expected identifier.",
            Error::E0052 => "Expected `(`.",
            Error::E0053 => "Expected identifier.",
            Error::E0054 => "Expected `,` or `)`.",
            Error::E0055 => "Tag names do not match.",
            Error::E0056 => "Expected `,` or `)`.",
            Error::E0057 => "Expected an identifier.",
            Error::E0058 => "Expected an identifier.",
            Error::E0059 => "Expected an identifier, `!` or `fn`.",
            Error::E0060 => "Expected `(`.",
            Error::E0061 => "Expected `{`.",
            Error::E0062 => "Expected `{`.",
            Error::E0063 => "Expected `if` or `{`.",
            Error::E0064 => "Expected `{`.",
            Error::E0065 => "Expected `{`.",
            Error::E0066 => "Unexpected token.",
            Error::E0067 => "Invalid assignment target. It must be an an identifier or an access expression.",
            Error::E0068 => "Expected an identifier.",
            Error::E0069 => "Expected `)` or `,`.",
            Error::E0070 => "Unexpected token.",
            Error::E0071 => "Expected an identifier.",
            Error::E0072 => "Missing `}`.",
            Error::E0073 => "Expected `{` or `;`.",
            Error::E0074 => "Expected an identifier.",
            Error::E0075 => "Expected `{` or `->`.",
            Error::E0076 => "Expected `(`.",
            Error::E0077 => "Expected `{`.",
            Error::E0078 => "Unexpected token",
        }
    }
    
    pub fn code_str(self) -> &'static str {
        match self {
            Error::E0000 => "E0000",
            Error::E0001 => "E0001",
            Error::E0003 => "E0003",
            Error::E0005 => "E0005",
            Error::E0007 => "E0007",
            Error::E0009 => "E0009",
            Error::E0011 => "E0011",
            Error::E0013 => "E0013",
            Error::E0014 => "E0014",
            Error::E0015 => "E0015",
            Error::E0016 => "E0016",
            Error::E0017 => "E0017",
            Error::E0018 => "E0018",
            Error::E0019 => "E0019",
            Error::E0020 => "E0020",
            Error::E0021 => "E0021",
            Error::E0022 => "E0022",
            Error::E0023 => "E0023",
            Error::E0024 => "E0024",
            Error::E0025 => "E0025",
            Error::E0026 => "E0026",
            Error::E0027 => "E0027",
            Error::E0028 => "E0028",
            Error::E0029 => "E0029",
            Error::E0030 => "E0030",
            Error::E0031 => "E0031",
            Error::E0032 => "E0032",
            Error::E0033 => "E0033",
            Error::E0034 => "E0034",
            Error::E0035 => "E0035",
            Error::E0036 => "E0036",
            Error::E0037 => "E0037",
            Error::E0038 => "E0038",
            Error::E0039 => "E0039",
            Error::E0040 => "E0040",
            Error::E0041 => "E0041",
            Error::E0042 => "E0042",
            Error::E0043 => "E0043",
            Error::E0044 => "E0044",
            Error::E0045 => "E0045",
            Error::E0046 => "E0046",
            Error::E0047 => "E0047",
            Error::E0048 => "E0048",
            Error::E0049 => "E0049",
            Error::E0050 => "E0050",
            Error::E0051 => "E0051",
            Error::E0052 => "E0052",
            Error::E0053 => "E0053",
            Error::E0054 => "E0054",
            Error::E0055 => "E0055",
            Error::E0056 => "E0056",
            Error::E0057 => "E0057",
            Error::E0058 => "E0058",
            Error::E0059 => "E0059",
            Error::E0060 => "E0060",
            Error::E0061 => "E0061",
            Error::E0062 => "E0062",
            Error::E0063 => "E0063",
            Error::E0064 => "E0064",
            Error::E0065 => "E0065",
            Error::E0066 => "E0066",
            Error::E0067 => "E0067",
            Error::E0068 => "E0068",
            Error::E0069 => "E0069",
            Error::E0070 => "E0070",
            Error::E0071 => "E0071",
            Error::E0072 => "E0072",
            Error::E0073 => "E0073",
            Error::E0074 => "E0074",
            Error::E0075 => "E0075",
            Error::E0076 => "E0076",
            Error::E0077 => "E0077",
            Error::E0078 => "E0078",
        }
    }

    pub const fn context(self) -> Option<&'static str> {
        match self {
            Error::E0000 => None,
            Error::E0001 => Some("in a two byte UTF-8 code point encoding."),
            Error::E0003 => Some("in a three byte UTF-8 code point encoding."),
            Error::E0005 => Some("in a three byte UTF-8 code point encoding."),
            Error::E0007 => Some("in a four byte UTF-8 code point encoding."),
            Error::E0009 => Some("in a four byte UTF-8 code point encoding."),
            Error::E0011 => Some("in a four byte UTF-8 code point encoding."),
            Error::E0013 => None,
            Error::E0014 => Some("after `\\\\` in a string escape."),
            Error::E0015 => Some("in a markup start tag."),
            Error::E0016 => Some("in a markup end tag."),
            Error::E0017 => Some("while lexing a decimal number literal."),
            Error::E0018 => Some("in a string literal."),
            Error::E0019 => Some("in a string literal."),
            Error::E0020 => Some("while lexing an identifier."),
            Error::E0021 => Some("in the first digit of a string literal hex escape code."),
            Error::E0022 => Some("in the first digit of a string literal hex escape code."),
            Error::E0023 => Some("in the first digit of a string literal hex escape code."),
            Error::E0024 => Some("in the second digit of a string literal hex escape code."),
            Error::E0025 => Some("in the second digit of a string literal hex escape code."),
            Error::E0026 => Some("in a markup end tag."),
            Error::E0027 => None,
            Error::E0028 => Some("while trying to construct a new token."),
            Error::E0029 => Some("while lexing the tail of a floating point decimal number."),
            Error::E0030 => Some("while lexing a character literal."),
            Error::E0031 => Some("while lexing a character literal."),
            Error::E0032 => Some("while lexing a character literal."),
            Error::E0033 => Some("in a self-closing markup tag."),
            Error::E0034 => Some("while collecting attributes in a markup start tag."),
            Error::E0035 => Some("while lexing an attribute in a markup start tag."),
            Error::E0036 => Some("while lexing a value in a markup start tag."),
            Error::E0037 => Some("in a markup element (while lexing children)."),
            Error::E0038 => Some("in a markup element (while lexing children)."),
            Error::E0039 => Some("while parsing a statement in a module."),
            Error::E0040 => Some("in type parameters."),
            Error::E0041 => Some("while parsing a mandatory statement because of previous annotations."),
            Error::E0042 => Some("while parsing an annotation."),
            Error::E0043 => Some("in `use`-statement children."),
            Error::E0044 => Some("after an identifier in a `use`-statement."),
            Error::E0045 => Some("in a `use`-statement path."),
            Error::E0046 => Some("in a `use`-statement."),
            Error::E0047 => Some("in a root `use`-statement."),
            Error::E0048 => Some("after `use`."),
            Error::E0049 => Some("at the end of a block."),
            Error::E0050 => Some("in a declaration."),
            Error::E0051 => Some("in a `struct`-statement."),
            Error::E0052 => Some("in a `struct`-statement. If you were trying to declare type parameters, move them right after the `struct` keyword."),
            Error::E0053 => Some("in a struct field declaration."),
            Error::E0054 => Some("After a struct field declaration."),
            Error::E0055 => Some("while parsing a markup end tag."),
            Error::E0056 => Some("in function parameters, after `this`."),
            Error::E0057 => Some("in a function parameter."),
            Error::E0058 => Some("in a type path."),
            Error::E0059 => Some("while parsing a type."),
            Error::E0060 => Some("while parsing a function expression."),
            Error::E0061 => Some("while parsing the body of a function expression. If you would like this function expression to be bodyless, remove the return type."),
            Error::E0062 => Some("while parsing the body of an if-expression. Help: Vine uses Rust-like syntax for if-expressions, rather than C- or Java-like syntax."),
            Error::E0063 => Some("in an else-chain."),
            Error::E0064 => Some("while parsing the body of an else-if-chain."),
            Error::E0065 => Some("while a while-expression."),
            Error::E0066 => Some("while parsing the beginning of an expression."),
            Error::E0067 => Some("while parsing an assignment expression"),
            Error::E0068 => Some("in an access expression."),
            Error::E0069 => Some("in a call expression."),
            Error::E0070 => Some("while parsing an expression extension."),
            Error::E0071 => Some("in a module declaration."),
            Error::E0072 => Some("after the module content."),
            Error::E0073 => Some("in a module declaration"),
            Error::E0074 => Some("in a function statement, after the `fn` keyword and type parameters."),
            Error::E0075 => Some("in a function statement. Help: if you want to use a bodyless function, split the function statement into declaration and expression."),
            Error::E0076 => Some("in a function statement (parameters)."),
            Error::E0077 => Some("in a function statement. After a return type, there must always be a braced function body."),
            Error::E0078 => None,
        }
    }

    pub const fn source(self) -> Source {
        match self {
            Error::E0000 => Source::UTF8,
            Error::E0001 => Source::UTF8,
            Error::E0003 => Source::UTF8,
            Error::E0005 => Source::UTF8,
            Error::E0007 => Source::UTF8,
            Error::E0009 => Source::UTF8,
            Error::E0011 => Source::UTF8,
            Error::E0013 => Source::Lexer,
            Error::E0014 => Source::Lexer,
            Error::E0015 => Source::Lexer,
            Error::E0016 => Source::Lexer,
            Error::E0017 => Source::Lexer,
            Error::E0018 => Source::Lexer,
            Error::E0019 => Source::Lexer,
            Error::E0020 => Source::Lexer,
            Error::E0021 => Source::Lexer,
            Error::E0022 => Source::Lexer,
            Error::E0023 => Source::Lexer,
            Error::E0024 => Source::Lexer,
            Error::E0025 => Source::Lexer,
            Error::E0026 => Source::Lexer,
            Error::E0027 => Source::Lexer,
            Error::E0028 => Source::Lexer,
            Error::E0029 => Source::Lexer,
            Error::E0030 => Source::Lexer,
            Error::E0031 => Source::Lexer,
            Error::E0032 => Source::Lexer,
            Error::E0033 => Source::Lexer,
            Error::E0034 => Source::Lexer,
            Error::E0035 => Source::Lexer,
            Error::E0036 => Source::Lexer,
            Error::E0037 => Source::Lexer,
            Error::E0038 => Source::Lexer,
            Error::E0039 => Source::Parser,
            Error::E0040 => Source::Parser,
            Error::E0041 => Source::Parser,
            Error::E0042 => Source::Parser,
            Error::E0043 => Source::Parser,
            Error::E0044 => Source::Parser,
            Error::E0045 => Source::Parser,
            Error::E0046 => Source::Parser,
            Error::E0047 => Source::Parser,
            Error::E0048 => Source::Parser,
            Error::E0049 => Source::Parser,
            Error::E0050 => Source::Parser,
            Error::E0051 => Source::Parser,
            Error::E0052 => Source::Parser,
            Error::E0053 => Source::Parser,
            Error::E0054 => Source::Parser,
            Error::E0055 => Source::Parser,
            Error::E0056 => Source::Parser,
            Error::E0057 => Source::Parser,
            Error::E0058 => Source::Parser,
            Error::E0059 => Source::Parser,
            Error::E0060 => Source::Parser,
            Error::E0061 => Source::Parser,
            Error::E0062 => Source::Parser,
            Error::E0063 => Source::Parser,
            Error::E0064 => Source::Parser,
            Error::E0065 => Source::Parser,
            Error::E0066 => Source::Parser,
            Error::E0067 => Source::Parser,
            Error::E0068 => Source::Parser,
            Error::E0069 => Source::Parser,
            Error::E0070 => Source::Parser,
            Error::E0071 => Source::Parser,
            Error::E0072 => Source::Parser,
            Error::E0073 => Source::Parser,
            Error::E0074 => Source::Parser,
            Error::E0075 => Source::Parser,
            Error::E0076 => Source::Parser,
            Error::E0077 => Source::Parser,
            Error::E0078 => Source::Parser,
        }
    }
}