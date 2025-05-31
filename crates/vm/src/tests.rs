#![cfg(test)]

use crate::gc::GC;
use crate::Value;

#[test]
fn gc() {
    let gc = GC::new(&[2]);
    let value = gc.allocate(0);
    value.lock()[0] = Value::from(1_u8);

    assert_eq!(
        &value.lock() as &[Value],
        &[Value::from(1_u8), Value::from(0_u8)]
    );

    assert_eq!(gc.count(), 1);
    gc.mark_and_sweep([].iter().copied());

    // `value` is invalid here

    assert_eq!(gc.count(), 0);
}
