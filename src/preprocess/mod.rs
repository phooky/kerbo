use std::cmp::{PartialEq,Ord};
use std::ops::Sub;
use std::num::Zero;

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

use std::iter::{Map,Zip};


macro_rules! print_err {
    ($($arg:tt)*) => (
        {
            use std::io::prelude::*;
            if let Err(e) = write!(&mut ::std::io::stderr(), "{}\n", format_args!($($arg)*)) {
                panic!("Failed to write to stderr.\
                    \nOriginal error output: {}\
                    \nSecondary error writing to stderr: {}", format!($($arg)*), e);
            }
        }
    )
}


pub struct SubtractIter<I,J> where
    I : Iterator<Item = J>,
    J : Sub<Output=J>+Zero+Ord
    
{
    minuend : I,
    subtrahend : I
}

impl<I,J> Iterator for SubtractIter<I,J> where
    I : Iterator<Item = J>,
    J : Sub<Output=J>+Zero+Ord
{
    type Item = J;
    fn next(&mut self) -> Option<J> {
        let m = self.minuend.next();
        let s = self.subtrahend.next();
        match m {
            None => None,
            Some(m) => match s {
                None => None,
                Some(s) => Some(if s > m { J::zero() } else { m - s })
            }
        }
    }
}

pub fn subtract<I,J>(minuend : I, subtrahend :  I)->SubtractIter<I,J> where
    I : Iterator<Item=J>,
    J : Sub<Output=J>+Zero+Ord
{
    SubtractIter{ minuend : minuend, subtrahend : subtrahend }
}

#[test]
fn test_subtract_u8() {
    let d1 = [10 as u8, 9, 8, 7, 6, 5];
    let d2 = [4 as u8, 5, 6, 7, 8, 9];
    //    let mut i =  d1.iter().zip(d2.iter()).map(|(x,y)| if y>x {u8::zero()} else {x-y});
    let mut i = subtract(d1.iter(),d2.iter());
    // assert_eq!(i.next(), Some(6));
    // assert_eq!(i.next(), Some(4));
    // assert_eq!(i.next(), Some(2));
    // assert_eq!(i.next(), Some(0));
    // assert_eq!(i.next(), Some(0));
    // assert_eq!(i.next(), Some(0));
    // assert_eq!(i.next(), None);
}
