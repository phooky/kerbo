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

