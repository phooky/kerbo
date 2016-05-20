
use std::ops::Index;

type Pixel = u8;

trait Image<P> : Index<(usize,usize),Output=P> {
    fn origin(&self) -> (usize, usize);
    fn size(&self) -> (usize, usize);
    fn sub_image(&self, origin : (usize, usize), size : (usize, usize)) -> &Image<P,Output=P>;
}

struct MemImage<'a,P:'a> {
    size : (usize, usize),
    data : &'a [P],
    stride : usize,
}

impl<'a,P> Image<P> for MemImage<'a,P> {
    fn origin(&self) -> (usize, usize) { (0,0) }
    fn size(&self) -> (usize, usize) { self.size }
    fn sub_image(&self, origin:(usize,usize), size:(usize,usize)) -> &Image<P,Output=P> {
        let (ox, oy) = origin;
        let offset = oy * self.stride + ox;
        &SliceImage { size : size,
                    data : &self.data[offset..],
                    origin : origin,
                    stride : self.stride } }
}

impl<'a,P> Index<(usize,usize)> for MemImage<'a,P> {
    type Output = P;
    fn index<'b>(&'b self, location : (usize,usize)) -> &'b P {
        let (x,y) = location;
        let offset = y * self.stride + x;
        self.data[offset]
    }
}

struct SliceImage<'a,P:'a> {
    size : (usize, usize),
    data : &'a [P],
    origin : (usize, usize),
    stride : usize,
}

impl<'a,P> Image<P> for SliceImage<'a,P> {
    fn origin(&self) -> (usize, usize) { self.origin }
    fn size(&self) -> (usize, usize) { self.size }
    fn sub_image(&self, origin:(usize,usize), size:(usize,usize)) -> &Image<P,Output=P> {
        let (ox, oy) = origin;
        let offset = oy * self.stride + ox;
        &SliceImage { size : size,
                    data : &self.data[offset..],
                    origin : origin,
                    stride : self.stride } }
}

impl<'a,P> Index<(usize,usize)> for SliceImage<'a,P> {
    type Output = P;
    fn index<'b>(&'b self, location : (usize,usize)) -> &'b P {
        let (x,y) = location;
        let offset = y * self.stride + x;
        self.data[offset]
    }
}

//impl<P> Image {
//    pub fn sub_image(&self, origin : (usize, usize), size : (usize, usize)