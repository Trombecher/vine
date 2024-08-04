#![cfg(test)]

use crate::bytes::Cursor;

#[test]
fn next() {
    let mut cursor = Cursor::new("AB".as_bytes());
    assert_eq!(cursor.next(), Some(b'A'));
    assert_eq!(cursor.next(), Some(b'B'));
    assert_eq!(cursor.next(), None);
}

#[test]
fn next_lfn() {
    let mut cursor = Cursor::new("A\r\nB\nC\rD".as_bytes());
    assert_eq!(cursor.next_lfn(), Some(b'A'));
    assert_eq!(cursor.next_lfn(), Some(b'\n'));
    assert_eq!(cursor.next_lfn(), Some(b'B'));
    assert_eq!(cursor.next_lfn(), Some(b'\n'));
    assert_eq!(cursor.next_lfn(), Some(b'C'));
    assert_eq!(cursor.next_lfn(), Some(b'\n'));
    assert_eq!(cursor.next_lfn(), Some(b'D'));
    assert_eq!(cursor.next_lfn(), None);
    assert_eq!(cursor.next_lfn(), None);
}

#[test]
fn peek() {
    let mut cursor = Cursor::new("AB".as_bytes());
    assert_eq!(cursor.peek(), Some(b'A'));
    assert_eq!(cursor.peek(), Some(b'A'));
    
    unsafe { cursor.advance_unchecked() }
    assert_eq!(cursor.peek(), Some(b'B'));
    
    unsafe { cursor.advance_unchecked() }
    assert_eq!(cursor.peek(), None);
}

#[test]
fn has_next() {
    let mut cursor = Cursor::new("A".as_bytes());
    assert_eq!(cursor.has_next(), true);

    unsafe { cursor.advance_unchecked() }
    assert_eq!(cursor.has_next(), false);
}

#[test]
fn can_rewind() {
    let mut cursor = Cursor::new("A".as_bytes());
    assert_eq!(cursor.can_rewind(), false);

    assert_eq!(cursor.next(), Some(b'A'));
    assert_eq!(cursor.can_rewind(), true);
}

#[test]
fn rewind_lfn() {
    let mut cursor = Cursor::new("AB\r".as_bytes());
    cursor.advance(); // A

    cursor.rewind();
    assert_eq!(cursor.next(), Some(b'A'));

    let _ = cursor.next_lfn(); // B
    let _ = cursor.next_lfn(); // LF
    
    cursor.rewind_lfn();
    assert_eq!(cursor.next_lfn(), Some(b'\n'));
}

#[test]
fn advance_char() {
    let mut cursor = Cursor::new("AB€C".as_bytes());
    
    assert_eq!(cursor.advance_char(), Ok(()));
    assert_eq!(cursor.peek(), Some(b'B'));
    
    cursor.advance_char().unwrap();
    cursor.advance_char().unwrap();
    assert_eq!(cursor.peek(), Some(b'C'));
    
    cursor.advance_char().unwrap();
    assert_eq!(cursor.peek(), None);
}