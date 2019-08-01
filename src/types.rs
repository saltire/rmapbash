#[derive(Clone, Copy, Debug)]
pub struct Edges<T> {
    pub n: T,
    pub e: T,
    pub s: T,
    pub w: T,
}

impl <T> Edges<T> {
    pub fn contains(&self, point: &Pair<T>) -> bool where T: Ord {
        point.x >= self.w && point.x <= self.e && point.z >= self.n && point.z <= self.s
    }

    pub fn size(&self) -> Pair<usize> where T: std::ops::Sub + std::ops::Add {
        Pair {
            x: (self.e - self.w + 1) as usize,
            z: (self.s - self.n + 1) as usize,
        }
    }

    pub fn full(size: usize) -> Edges<usize> {
        Edges {
            n: 0,
            e: size - 1,
            s: size - 1,
            w: 0,
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Pair<T> {
    pub x: T,
    pub z: T,
}
