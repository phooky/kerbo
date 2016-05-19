
// All of our preprocessing iterators.

pub struct ChromaStripIter<I> where
    I : Iterator<Item = u8>
{
    i : I
}

impl<I> Iterator for ChromaStripIter<I> where
    I : Iterator<Item = u8>
{
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        let val = self.i.next();
        if val != None {
            self.i.next(); // discard
        }
        val
    }
}

/// Strip chroma information from a yuyv image, returning yy (4:2:2->4:0:0)
pub fn strip_chroma_yuyv<I>(source : I) -> ChromaStripIter<I> where
    I : Iterator<Item = u8>
{
    ChromaStripIter { i : source }
}
