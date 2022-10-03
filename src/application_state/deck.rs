use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Deck {
    id: String,
    card_paths: Vec<String>,
    interval_coefficients: IntervalCoefficients,
}

impl Deck {
    fn new (id: &str, card_paths: Vec<&str>, interval_coefficients: IntervalCoefficients) -> Self {
        Self{
            id: id.to_string(),
            card_paths: card_paths.iter().map(|s| s.to_string()).collect(),
            interval_coefficients
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize,)]
pub struct IntervalCoefficients {
    pub pass_coef: f64,
    pub easy_coef: f64,
    pub fail_coef: f64,
}

impl IntervalCoefficients {
    pub fn new(pass_coef: f64, easy_coef: f64, fail_coef: f64) -> Self {
        Self {
            pass_coef,
            easy_coef,
            fail_coef,
        }
    }
}

impl Default for IntervalCoefficients {
    fn default() -> Self {
        Self::new(1.0, 1.3, 0.0)
    }
}

#[cfg(test)]
mod unit_tests {

    use super::*;

    mod deck {

        use super::*;

        #[test]
        fn new() {
            let id = "cephelapods";
            let card_paths = vec!["octopus", "squid", "cuttlefish", "nautilus"];
            let expected_card_paths = vec![
                String::from("octopus"),
                String::from("squid"),
                String::from("cuttlefish"),
                String::from("nautilus")
            ];
            let interval_coefficients = IntervalCoefficients {
                pass_coef: 8.0,
                easy_coef: 9.0,
                fail_coef: 10.0,
            };
            let expected = Deck {
                id: id.to_string(),
                card_paths: expected_card_paths,
                interval_coefficients: interval_coefficients.clone()
            };
            let actual = Deck::new(id, card_paths, interval_coefficients);
            assert_eq!(expected, actual);
        }
    }

    mod interval_coefficients {

        use super::*;

        #[test]
        fn new() {
            let (pass_coef, easy_coef, fail_coef) = (1.0, 2.0, 3.0);
            let expected = IntervalCoefficients {
                pass_coef,
                easy_coef,
                fail_coef,
            };
            let actual = IntervalCoefficients::new(pass_coef, easy_coef, fail_coef);
            assert_eq!(expected, actual);
        }

        #[test]
        fn default() {
            let pass_coef = 1.0;
            let easy_coef = 1.3;
            let fail_coef = 0.0;
            let expected = IntervalCoefficients {
                pass_coef,
                easy_coef,
                fail_coef
            };
            let actual = IntervalCoefficients::default();
            assert_eq!(expected, actual);
        }
    }
}
