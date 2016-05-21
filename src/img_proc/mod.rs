
use std::ops::Sub;
use std::clone::Clone;

type Pixel = u8;

struct MemImage<P> {
    size : (usize, usize),
    data : Vec<P>,
    stride : usize,
}

impl<P : Sub<P,Output=P> + Clone > MemImage<P> {
    fn subtract(&mut self, other : MemImage<P>) {
        let mut o = other.data.iter();
        for p in self.data.iter_mut() {
            match o.next() {
                Some(d) => {
                    let v = p.clone().sub(d.clone());
                    *p = v;
                },
                None => break,
            }
        }
    }
}

#[test]
fn test_image_sub() {
    let mut a = MemImage{ size: (3, 3),
                     data : vec![9, 8, 7, 8, 7, 6, 7, 6, 5],
                     stride : 3 };
    let b = MemImage{ size: (3, 3),
                     data : vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                     stride : 3 };
    a.subtract(b);
    assert_eq!( a.data, vec![8,6,4,4,2,0,0,-2,-4]);
}