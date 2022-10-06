pub trait Identifiable {
    fn uid(&self) -> &str;
}

pub trait ProtectedField<T> {
    fn with_protected_field(self, other: &T) -> Self;
}
