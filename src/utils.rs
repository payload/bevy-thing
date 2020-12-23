pub trait SliceExt<T> {
    fn random(&self) -> T;
}

impl<T: Copy> SliceExt<T> for [T] {
    fn random(&self) -> T {
        self[rand::random::<usize>() % self.len()]
    }
}
