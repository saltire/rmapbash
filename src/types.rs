pub struct Edges<T> {
    pub n: T,
    pub e: T,
    pub s: T,
    pub w: T,
}

#[derive(Eq, Hash, PartialEq)]
pub struct Pair<T> {
    pub x: T,
    pub z: T,
}
