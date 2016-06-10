use std::cmp::PartialEq;

// All of our preprocessing iterators.

pub struct ChromaStripIter<I,J> where
    I : Iterator<Item = J>,
    J : PartialEq
{
    i : I
}

impl<I,J> Iterator for ChromaStripIter<I,J> where
    I : Iterator<Item = J>,
    J : PartialEq
{
    type Item = J;
    fn next(&mut self) -> Option<J> {
        let val = self.i.next();
        if val != None {
            self.i.next(); // discard
        }
        val
    }
}

/// Strip chroma information from a yuyv image, returning yy (4:2:2->4:0:0)
pub fn strip_chroma_yuyv<I,J>(source : I) -> ChromaStripIter<I,J> where
    I : Iterator<Item = J>,
    J : PartialEq
{
    ChromaStripIter { i : source }
}

use std::io::{stderr,Write};

#[test]
fn test_chroma_stripper_u8() {
    let data = vec![1 as u8, 2, 1, 2, 1, 2, 1, 2];
    let mut i = strip_chroma_yuyv(data.iter());
    assert_eq!(i.next(), Some(&1));
    assert_eq!(i.next(), Some(&1));
    assert_eq!(i.next(), Some(&1));
    assert_eq!(i.next(), Some(&1));
    assert_eq!(i.next(), None);
}

#[test]
fn test_chroma_stripper_i32() {
    let data = vec![1 as i32, 2, 1, 2, 1, 2, 1, 2];
    let mut i = strip_chroma_yuyv(data.iter());
    assert_eq!(i.next(), Some(&1));
    assert_eq!(i.next(), Some(&1));
    assert_eq!(i.next(), Some(&1));
    assert_eq!(i.next(), Some(&1));
    assert_eq!(i.next(), None);
}

