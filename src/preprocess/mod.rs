use std::cmp::{PartialEq,Ord};
use std::ops::Sub;
use std::num::Zero;
use std::marker::PhantomData;

// All of our preprocessing iterators.

pub struct ChromaStripIter<'a,I,J> where
    I : Iterator<Item = &'a J>,
    J : PartialEq+'a
{
    i : I,
    marker : PhantomData<&'a J>
}

impl<'a,I,J> Iterator for ChromaStripIter<'a,I,J> where
    I : Iterator<Item = &'a J>,
    J : PartialEq+'a
{
    type Item = &'a J;
    fn next(&mut self) -> Option<&'a J> {
        let val = self.i.next();
        if val != None {
            self.i.next(); // discard
        }
        val
    }
}

/// Strip chroma information from a yuyv image, returning yy (4:2:2->4:0:0)
pub fn strip_chroma_yuyv<'a,I,J>(source : I) -> ChromaStripIter<'a,I,J> where
    I : Iterator<Item = &'a J>,
    J : PartialEq+'a
{
    ChromaStripIter { i : source, marker : PhantomData }
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

pub struct SubtractIter<'a,I,J> where
    I : Iterator<Item = &'a J>+'a,
    J : Sub<Output=J>+Zero+Ord+Clone+'a    
{
    minuend : I,
    subtrahend : I,
    marker : PhantomData<&'a J>    
}

impl<'a,I,J> Iterator for SubtractIter<'a,I,J> where
    I : Iterator<Item = &'a J>,
    J : Sub<Output=J>+Zero+Ord+Clone+'a
{
    type Item = J;
    fn next(&mut self) -> Option<J> {
        let m = self.minuend.next();
        let s = self.subtrahend.next();
        match m {
            None => None,
            Some(m) => match s {
                None => None,
                Some(s) => Some(if s > m { J::zero() } else { m.clone() - s.clone() })
            }
        }
    }
}

pub fn subtract<'a,I,J>(minuend : I, subtrahend : I)->SubtractIter<'a,I,J> where
    I : Iterator<Item=&'a J>,
    J : Sub<Output=J>+Zero+Ord+Clone+'a
{
    SubtractIter{ minuend : minuend,
                  subtrahend : subtrahend,
                  marker : PhantomData }
}

#[test]
fn test_subtract_u8() {
    let d1 = vec![10 as u8, 9, 8, 7, 6, 5];
    let d2 = vec![4 as u8, 5, 6, 7, 8, 9];
    //    let mut i =  d1.iter().zip(d2.iter()).map(|(x,y)| if y>x {u8::zero()} else {x-y});
    let mut i = subtract(d1.iter(),d2.iter());
    assert_eq!(i.next(), Some(6));
    assert_eq!(i.next(), Some(4));
    assert_eq!(i.next(), Some(2));
    assert_eq!(i.next(), Some(0));
    assert_eq!(i.next(), Some(0));
    assert_eq!(i.next(), Some(0));
    assert_eq!(i.next(), None);
}


#[test]
fn test_subtract_i32() {
    let d1 = vec![10 as i32, 9, 8, 7, 6, 5];
    let d2 = vec![4 as i32, 5, 6, 7, 8, 9];
    //    let mut i =  d1.iter().zip(d2.iter()).map(|(x,y)| if y>x {u8::zero()} else {x-y});
    let mut i = subtract(d1.iter(),d2.iter());
    assert_eq!(i.next(), Some(6));
    assert_eq!(i.next(), Some(4));
    assert_eq!(i.next(), Some(2));
    assert_eq!(i.next(), Some(0));
    assert_eq!(i.next(), Some(0));
    assert_eq!(i.next(), Some(0));
    assert_eq!(i.next(), None);
}
