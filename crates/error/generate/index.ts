import {write} from "bun";
import {type RawError, RAW_ERRORS, SOURCES} from "./error_codes";

type ErrorCode = RawError & {
    format: string,
    code: number,
};

const formattedCodes = Object
    .entries(RAW_ERRORS)
    .map<ErrorCode>(([code, rawError]) => ({
        ...rawError,
        code: +code,
        format: `E${code.padStart(4, "0")}`
    }));

await write("../src/generated_codes.rs", `//! This file was automatically generated.

#[derive(PartialEq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum Error {${formattedCodes
    .map(error => `\n    /// ${error.description}${error.context ? `\n    ///\n    /// Context: The error occurred ${error.context}` : ""}\n    ${error.format} = ${error.code},`)
    .join("\n")}
}

#[derive(PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Source {${Object.values(SOURCES).map(source => `\n    ${source},`).join("")}
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {${Object.entries(SOURCES).map(([s, en]) => `\n            Source::${en} => "${s}",`).join("")}
        }
    }
}

impl Error {
    pub fn as_str(self) -> &'static str {
        match self {${
    formattedCodes
        .map(error =>
            `\n            Error::${error.format} => ${JSON.stringify(error.description)},`)
        .join("")
}
        }
    }
    
    pub fn code_str(self) -> &'static str {
        match self {${formattedCodes.map(error => `\n            Error::${error.format} => "${error.format}",`).join("")}
        }
    }

    pub const fn context(self) -> Option<&'static str> {
        match self {${formattedCodes.map(error => `\n            Error::${error.format} => ${error.context ? `Some(${JSON.stringify(error.context)})` : "None"},`).join("")}
        }
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
}`);