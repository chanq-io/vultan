pub trait UID {
    fn uid(&self) -> &str;
}

pub trait Merge<T> {
    fn merge(self, other: &T) -> Self;
}
