#![cfg(test)]

use crate::{Cursor, Error};

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
fn has_next() {
    let mut cursor = Cursor::new("A".as_bytes());
    assert_eq!(cursor.has_next(), true);

    unsafe { cursor.advance_unchecked() }
    assert_eq!(cursor.has_next(), false);
}

#[test]
fn peek() {
    let mut cursor = Cursor::new(b"AB");
    assert_eq!(cursor.peek(), Some(b'A'));
    assert_eq!(cursor.peek(), Some(b'A'));
    
    unsafe { cursor.advance_unchecked() }
    assert_eq!(cursor.peek(), Some(b'B'));
    
    unsafe { cursor.advance_unchecked() }
    assert_eq!(cursor.peek(), None);
}

#[test]
fn peek_n() {
    let mut cursor = Cursor::new(b"AB");
    assert_eq!(cursor.peek_n(0), Some(b'A'));
    assert_eq!(cursor.peek_n(1), Some(b'B'));
    assert_eq!(cursor.peek_n(2), None);
}

#[test]
fn advance() {
    let mut cursor = Cursor::new(b"AB");

    cursor.advance();
    assert_eq!(cursor.peek(), Some(b'B'));

    cursor.advance();
    assert_eq!(cursor.peek(), None);

    cursor.advance();
    assert_eq!(cursor.peek(), None);
}

#[test]
fn advance_char() {
    let mut cursor = Cursor::new("AB€C".as_bytes());
    
    assert_eq!(cursor.advance_char(), Ok(()));
    assert_eq!(cursor.peek(), Some(b'B'));

    assert_eq!(cursor.advance_char(), Ok(()));
    assert_eq!(cursor.advance_char(), Ok(()));
    assert_eq!(cursor.peek(), Some(b'C'));

    assert_eq!(cursor.advance_char(), Ok(()));
    assert_eq!(cursor.peek(), None);
}

#[test]
fn advance_char_error() {
    assert_eq!(Cursor::new(b"\x80").advance_char(), Err(Error::EncounteredContinuationByte));

    assert_eq!(Cursor::new(b"\xC2").advance_char(), Err(Error::Missing2ndOf2));
    assert_eq!(Cursor::new(b"\xC2\x00").advance_char(), Err(Error::Invalid2ndOf2));

    assert_eq!(Cursor::new(b"\xE0").advance_char(), Err(Error::Missing2ndOf3));
    assert_eq!(Cursor::new(b"\xE0\x00").advance_char(), Err(Error::Invalid2ndOf3));
    assert_eq!(Cursor::new(b"\xE0\x80").advance_char(), Err(Error::Missing3rdOf3));
    assert_eq!(Cursor::new(b"\xE0\x80\x00").advance_char(), Err(Error::Invalid3rdOf3));

    assert_eq!(Cursor::new(b"\xF0").advance_char(), Err(Error::Missing2ndOf4));
    assert_eq!(Cursor::new(b"\xF0\x00").advance_char(), Err(Error::Invalid2ndOf4));
    assert_eq!(Cursor::new(b"\xF0\x80").advance_char(), Err(Error::Missing3rdOf4));
    assert_eq!(Cursor::new(b"\xF0\x80\x00").advance_char(), Err(Error::Invalid3rdOf4));
    assert_eq!(Cursor::new(b"\xF0\x80\x80").advance_char(), Err(Error::Missing4thOf4));
    assert_eq!(Cursor::new(b"\xF0\x80\x80\x00").advance_char(), Err(Error::Invalid4thOf4));
}

#[test]
fn skip_ascii_whitespace() {
    let mut cursor = Cursor::new(b" \r\n\t\x0CB");

    cursor.skip_ascii_whitespace();
    assert_eq!(cursor.peek(), Some(b'B'));
}