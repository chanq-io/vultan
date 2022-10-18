use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
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
            fail_coef,
        };
        let actual = IntervalCoefficients::default();
        assert_eq!(expected, actual);
    }
}
