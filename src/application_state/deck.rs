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
