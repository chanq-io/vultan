pub trait UID {
    fn uid(&self) -> &str;
}

pub trait Merge<T> {
    fn merge(self, other: &T) -> Self;
}

#[cfg(test)]
pub mod test_tools {

    use super::*;
    use std::collections::HashMap;

    #[derive(Debug)]
    pub enum Expect<T> {
        DoesContain(T),
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
                })
                .count();
            assert!(container.len() == expected_length);
        }

        pub fn assert_uid_map_contains<'a, T>(map: &HashMap<String, T>, expected: &'a [Expect<T>])
        where
            T: Default + std::fmt::Debug + PartialEq + UID,
        {
            assert_length_matches(map, expected);
            for comparator in expected.iter() {
                match comparator {
                    Expect::DoesContain(item) => assert!(uid_map_contains(map, item)),
                    Expect::DoesNotContain(item) => assert!(!uid_map_contains(map, item)),
                    _ => panic!("BAD TEST"),
                }
            }
        }
    }
}
