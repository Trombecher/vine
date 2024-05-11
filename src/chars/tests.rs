#![cfg(test)]

use crate::chars::Cursor;

#[test]
fn next() {
    let mut cursor = Cursor::new("AB".as_bytes());
    assert_eq!(cursor.next(), Some(b'A'));
    assert_eq!(cursor.next(), Some(b'B'));
    assert_eq!(cursor.next(), None);
}

#[test]
fn peek_raw() {
    let mut cursor = Cursor::new("AB".as_bytes());
    assert_eq!(cursor.peek_raw(), Some(b'A'));
    assert_eq!(cursor.peek_raw(), Some(b'A'));
    
    unsafe { cursor.advance_unchecked() }
    assert_eq!(cursor.peek_raw(), Some(b'B'));
    
    unsafe { cursor.advance_unchecked() }
    assert_eq!(cursor.peek_raw(), None);
}

#[test]
fn advance_char() {
    let mut cursor = Cursor::new("AB€C".as_bytes());
    
    cursor.advance_char().unwrap();
    assert_eq!(cursor.peek_raw(), Some(b'B'));
    
    cursor.advance_char().unwrap();
    cursor.advance_char().unwrap();
    assert_eq!(cursor.peek_raw(), Some(b'C'));
    
    cursor.advance_char().unwrap();
    assert_eq!(cursor.peek_raw(), None);
}