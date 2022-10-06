pub trait Identifiable<'a> {
    fn uid(&'a self) -> &'a str;
}
