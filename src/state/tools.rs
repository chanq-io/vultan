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
    pub enum ExpectContains<T> {
        Yes(T),
        No(T),
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

        pub fn assert_length_matches<'a, C, T>(container: &C, expected: &Vec<ExpectContains<T>>)
        where
            C: ?Sized + Len,
            T: Default,
        {
            let expected_length = expected
                .iter()
                .filter(|c| {
                    std::mem::discriminant(*c)
                        == std::mem::discriminant(&ExpectContains::Yes(T::default()))
                })
                .count();
            assert!(container.len() == expected_length);
        }

        pub fn assert_uid_map_contains<'a, T>(
            map: &HashMap<String, T>,
            expected: &'a Vec<ExpectContains<T>>,
        ) where
            T: Default + std::fmt::Debug + PartialEq + UID,
        {
            assert_length_matches(map, expected);
            for comparator in expected.iter() {
                match comparator {
                    ExpectContains::Yes(item) => assert!(uid_map_contains(map, item)),
                    ExpectContains::No(item) => assert!(!uid_map_contains(map, item)),
                }
            }
        }
    }
}
