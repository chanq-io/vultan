pub struct Deck<'a> {
    tag: String,
    card_paths: Vec<&'a str>,
    exponential_backoff_settings: IntervalCoefficients,
}

#[derive(Clone, Debug, PartialEq)]
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
            fail_coef
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn new_interval_coefficients() {
        let (pass_coef, easy_coef, fail_coef) = (1.0, 2.0, 3.0);
        let expected = IntervalCoefficients{
            pass_coef,
            easy_coef,
            fail_coef
        };
        let actual = IntervalCoefficients::new(pass_coef, easy_coef, fail_coef);
        assert_eq!(expected, actual);
    }
}
