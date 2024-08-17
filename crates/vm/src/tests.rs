#![cfg(test)]

use crate::gc::GC;
use crate::Value;

#[test]
fn object_creation_and_access() {
    let mut gc = GC::new(&[4]);
    let obj = gc.allocate_object(0).unwrap();

    for (i, entry) in obj.lock().as_mut_slice().iter_mut().enumerate() {
        *entry = Value::from(i as u64);
    }
    
    use std::io::Write;
    
    let mut collect = Vec::new();
    write!(collect, "{:?}", obj).unwrap();
    
    assert_eq!(collect.as_slice(), b"Object(Raw(0), Raw(1), Raw(2), Raw(3))");
}

#[test]
#[should_panic]
fn locks() {
    let mut gc = GC::new(&[4]);
    
}