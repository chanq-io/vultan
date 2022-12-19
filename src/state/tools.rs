#[cfg(test)]
use mockall::mock;
pub trait UID {
    fn uid(&self) -> &str;
}

pub trait Merge<T> {
    fn merge(self, other: &T) -> Self;
}

pub trait Near<T> {
    fn is_near(&self, other: &T) -> bool;
}

pub trait IO {
    fn path(&self) -> &str;
    fn read(&self) -> Result<String, std::io::Error>;
    fn write(&self, content: String) -> Result<(), std::io::Error>;
}

#[cfg(test)]
pub mod test_tools {

    use super::*;
    use std::collections::HashMap;

    mock! {
    // Structure to mock
    pub IO {}
    // First trait to implement on C
    impl IO for IO {
        fn path(&self) -> &str;
        fn read(&self) -> Result<String, std::io::Error>;
        fn write(&self, content: String) -> Result<(), std::io::Error>;
    } }

    pub fn mock_filesystem_reader(path: String) -> MockIO {
        let mut handle = MockIO::new();
        let path = path.to_string();
        handle.expect_path().return_const(path.clone());
        handle
            .expect_read()
            .returning(move || std::fs::read_to_string(path.clone()));
        handle
    }

    pub fn mock_filesystem_writer(path: String) -> MockIO {
        let mut handle = MockIO::new();
        handle.expect_path().return_const(path.to_string());
        let path = path.to_string();
        handle.expect_write().returning(move |content: String| {
            std::fs::write(path.clone(), content.as_str())
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, ""))
        });
        handle
    }

    pub fn ignore<T: Default>() -> T {
        Default::default()
    }

    #[derive(Debug)]
    pub enum Expect<T> {
        DoesContain(T),
        DoesContainNear(T),
        DoesNotContain(T),
        Truthy,
        Falsy,
    }

    pub fn assert_truthy<T>(expectation: Expect<T>, value: bool) {
        assert!(match expectation {
            Expect::Truthy => value,
            Expect::Falsy => !value,
            _ => panic!("BAD TEST"),
        })
    }

    fn uid_map_contains<'a, T>(map: &HashMap<String, T>, item: &'a T) -> bool
    where
        T: PartialEq + UID,
    {
        map.contains_key(item.uid()) && *item == map[item.uid()]
    }

    fn uid_map_contains_near<'a, T>(map: &HashMap<String, T>, item: &'a T) -> bool
    where
        T: PartialEq + UID + Near<T>,
    {
        map.contains_key(item.uid()) && item.is_near(&map[item.uid()])
    }

    pub mod assertions {

        use super::*;
        use len_trait::Len;

        pub fn assert_length_matches<'a, C, T>(container: &C, expected: &[Expect<T>])
        where
            C: ?Sized + Len,
            T: Default,
        {
            let expected_length = expected
                .iter()
                .filter(|c| {
                    std::mem::discriminant(*c)
                        == std::mem::discriminant(&Expect::DoesContain(T::default()))
                        || std::mem::discriminant(*c)
                            == std::mem::discriminant(&Expect::DoesContainNear(T::default()))
                })
                .count();
            assert!(container.len() == expected_length);
        }

        pub fn assert_uid_map_contains<'a, T>(map: &HashMap<String, T>, expected: &'a [Expect<T>])
        where
            T: Default + std::fmt::Debug + PartialEq + UID + Near<T>,
        {
            assert_length_matches(map, expected);
            for comparator in expected.iter() {
                match comparator {
                    Expect::DoesContain(item) => assert!(uid_map_contains(map, item)),
                    Expect::DoesContainNear(item) => assert!(uid_map_contains_near(map, item)),
                    Expect::DoesNotContain(item) => assert!(!uid_map_contains(map, item)),
                    _ => panic!("BAD TEST"),
                }
            }
        }
    }
}
