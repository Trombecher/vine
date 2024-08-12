use js::refs::{Ref, RefGenerator};

fn main() {
    let iter = RefGenerator::new().map(|r| {
        let mut vec = Vec::new();
        r.write_into(&mut vec);
        (r, unsafe { String::from_utf8_unchecked(vec) })
    });

    for (r, vec) in iter.take(1000) {
        println!("{} => {vec:?} => {}", r.raw, Ref::from_encoded_iter(vec.as_bytes().iter().copied()).raw)
    }
}