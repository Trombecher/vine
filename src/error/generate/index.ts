import {write} from "bun";
import codes from "./error_codes.json";

type ErrorCode = {
    format: string,
    description: string,
    context: string | undefined,
    hint: keyof typeof HINTS | undefined,
    source: keyof typeof SOURCES,
}

const formattedCodes = Object
    .keys(codes)
    .toSorted((a, b) => +a - +b)
    .map<ErrorCode>(code => {
        // @ts-ignore
        const obj = codes["" + code] as any;

        obj.format = `E${code.toString().padStart(4, "0")}`;
        return obj;
    });

const SOURCES = {
    "utf-8": "UTF8",
    "lexer": "Lexer",
} as const;

const HINTS = {
    "unexpected end of input": "UnexpectedEndOfInput",
    "unexpected byte": "UnexpectedByte",
    "unexpected character": "UnexpectedCharacter"
} as const;

await write("../mod.rs", `//! This file was automatically generated.

use std::fmt::{Debug, Formatter};

#[derive(PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Error {${formattedCodes
    .map(error => `\n    /// ${error.description}${error.context ? `\n    ///\n    /// Context: The error occurred ${error.context}` : ""}\n    ${error.format},`)
    .join("\n")}
}

#[derive(PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Source {${Object.values(SOURCES).map(source => `\n    ${source},`).join("")}
}

#[derive(PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Hint {${Object.values(HINTS).map(hint => `\n    ${hint},`).join("")}
}

impl Hint {
    pub fn as_str(self) -> &'static str {
        match self {${
    Object
        .entries(HINTS)
        .map(([format, enumeration]) =>
            `\n            Hint::${enumeration} => "${format}",`)
        .join("")
}
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {${
    formattedCodes
        .map(error =>
            `\n            Error::${error.format} => f.write_str("${error.description}"),`)
        .join("")
}
        }
    }
}

impl Error {
    pub const fn code(self) -> u8 {
        self as u8
    }

    pub const fn source(self) -> Source {
        match self {${
    formattedCodes
        .map(error =>
            `\n            Error::${error.format} => Source::${SOURCES[error.source]},`)
        .join("")
}
        }
    }

    pub const fn hint(self) -> Option<Hint> {
        match self {${
    formattedCodes
        .map(error =>
            `\n            Error::${error.format} => ${error.hint ? `Some(Hint::${HINTS[error.hint]})` : "None"},`)
        .join("")
}
        }
    }
}`);