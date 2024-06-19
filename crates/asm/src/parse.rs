pub enum Keyword {
    Entry,
    Fn,
    Pub,
    Required,
    Feature,
    Instruction,
    Optional,
}

pub static KEYWORD_MAP: phf::Map<&'static str, Keyword> = phf::phf_map!(
    "entry" => Keyword::Entry,
    "fn" => Keyword::Fn,
    "pub" => Keyword::Pub,
    "required" => Keyword::Required,
    "feature" => Keyword::Feature,
    "instruction" => Keyword::Instruction,
    "optional" => Keyword::Optional,
);