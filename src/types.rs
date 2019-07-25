#[derive(Clone, Copy, Debug, Default)]
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
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Pair<T> {
    pub x: T,
    pub z: T,
}
