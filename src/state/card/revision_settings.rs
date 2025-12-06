use super::score::Score;
use crate::state::deck::IntervalCoefficients;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
pub struct RevisionSettings {
    pub due: DateTime<Utc>,
    pub interval: f64,
    pub memorisation_factor: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct IntervalCalculationSettings<'ics> {
    coefficients: &'ics IntervalCoefficients,
    days_overdue: f64,
}

#[derive(Debug, PartialEq)]
struct PossibleIntervals(f64, f64, f64, f64);

impl RevisionSettings {
    pub fn new(due: DateTime<Utc>, interval: f64, memorisation_factor: f64) -> Self {
        Self {
            due,
            interval,
            memorisation_factor,
        }
    }

    pub fn transform(self, score: Score, coefficients: &IntervalCoefficients) -> Self {
        let new_interval = self.calculate_new_interval(&score, coefficients);
        Self {
            due: self.calculate_new_due_date(new_interval),
            interval: new_interval,
            memorisation_factor: self.calculate_new_memorisation_factor(&score),
        }
    }

    fn calculate_new_due_date(&self, new_interval: f64) -> DateTime<Utc> {
        let seconds_in_minute = 60.0;
        let minutes_in_hour = 60.0;
        let hours_in_day = 24.0;
        let seconds_in_interval = seconds_in_minute * minutes_in_hour * hours_in_day * new_interval;
        self.due + Duration::seconds(seconds_in_interval as i64)
    }

    fn calculate_new_memorisation_factor(&self, score: &Score) -> f64 {
        let default_factor: f64 = 1300.0;
        match score {
            Score::Fail => default_factor.max(self.memorisation_factor - 200.0),
            Score::Hard => default_factor.max(self.memorisation_factor - 150.0),
            Score::Pass => default_factor.max(self.memorisation_factor),
            Score::Easy => default_factor.max(self.memorisation_factor + 150.0),
        }
    }

    fn calculate_new_interval(&self, score: &Score, coefficients: &IntervalCoefficients) -> f64 {
        let PossibleIntervals(fail_interval, hard_interval, pass_interval, easy_interval) =
            self.calculate_possible_intervals(coefficients);
        match score {
            Score::Fail => fail_interval,
            Score::Hard => hard_interval,
            Score::Pass => pass_interval,
            Score::Easy => easy_interval,
        }
    }

    fn calculate_possible_intervals(
        &self,
        coefficients: &IntervalCoefficients,
    ) -> PossibleIntervals {
        let calculation_settings = self.create_interval_calculation_settings(coefficients);
        let fail_interval = self.calculate_fail_interval(&calculation_settings);
        let hard_interval = self.calculate_hard_interval(&calculation_settings);
        let pass_interval = self.calculate_pass_interval(&calculation_settings, hard_interval);
        let easy_interval = self.calculate_easy_interval(&calculation_settings, pass_interval);
        PossibleIntervals(fail_interval, hard_interval, pass_interval, easy_interval)
    }

    fn create_interval_calculation_settings<'a>(
        &self,
        coefficients: &'a IntervalCoefficients,
    ) -> IntervalCalculationSettings<'a> {
        let present = Utc::now();
        let past = self.due;
        let days_overdue_quantised_by_hour =
            (present.signed_duration_since(past).num_hours() as f64) / 24.0;
        IntervalCalculationSettings {
            coefficients,
            days_overdue: days_overdue_quantised_by_hour,
        }
    }

    fn calculate_fail_interval(&self, calculation_settings: &IntervalCalculationSettings) -> f64 {
        self.interval * calculation_settings.coefficients.fail_coef
    }

    fn calculate_hard_interval(&self, calculation_settings: &IntervalCalculationSettings) -> f64 {
        let fallback = self.interval + 1.0;
        let hard_coef = 1.2;
        let base_num_days = self.interval + calculation_settings.days_overdue * 0.25;
        fallback.max(hard_coef * base_num_days * calculation_settings.coefficients.pass_coef)
    }

    fn calculate_pass_interval(
        &self,
        calculation_settings: &IntervalCalculationSettings,
        hard_interval: f64,
    ) -> f64 {
        let fallback = hard_interval + 1.0;
        let base_num_days = self.interval + calculation_settings.days_overdue * 0.5;
        let memorisation_coef = self.memorisation_factor * 0.001;
        let pass_coef = calculation_settings.coefficients.pass_coef;
        fallback.max(base_num_days * memorisation_coef * pass_coef)
    }

    fn calculate_easy_interval(
        &self,
        calculation_settings: &IntervalCalculationSettings,
        pass_interval: f64,
    ) -> f64 {
        let fallback = pass_interval + 1.0;
        let base_num_days = self.interval + calculation_settings.days_overdue;
        let memorisation_coef = self.memorisation_factor * 0.001;
        let pass_coef = calculation_settings.coefficients.pass_coef;
        let easy_coef = calculation_settings.coefficients.easy_coef;
        fallback.max(base_num_days * memorisation_coef * pass_coef * easy_coef)
    }
}

impl Default for RevisionSettings {
    fn default() -> Self {
        Self::new(Utc::now(), 0.0, 1300.0)
    }
}

#[cfg(test)]
pub mod assertions {
    use super::*;
    pub fn assert_revision_settings_near(
        a: &RevisionSettings,
        b: &RevisionSettings,
        due_difference_tolerance_in_seconds: i64,
    ) {
        assert_eq!(a.interval, b.interval);
        assert_eq!(a.memorisation_factor, b.memorisation_factor);
        assert!(
            a.due.signed_duration_since(b.due).num_seconds().abs()
                < due_difference_tolerance_in_seconds
        );
    }
}

pub mod test_tools {
    use super::*;

    pub fn make_expected_revision_settings(
        original_due_date: &DateTime<Utc>,
        interval: f64,
        factor: f64,
    ) -> RevisionSettings {
        RevisionSettings::new(
            original_due_date.to_owned() + duration_from_interval(interval),
            interval,
            factor,
        )
    }

    pub fn duration_from_interval(interval: f64) -> Duration {
        Duration::seconds((86400.0 * interval) as i64)
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use chrono::Duration;
    use rstest::*;

    fn make_interval_calculation_settings<'a>(
        coefficients: &'a IntervalCoefficients,
        days_overdue: f64,
    ) -> IntervalCalculationSettings<'a> {
        IntervalCalculationSettings {
            coefficients,
            days_overdue,
        }
    }

    #[test]
    fn new() {
        let due = Utc::now();
        let interval = 123.0;
        let memorisation_factor = 234.5;
        let expected = RevisionSettings {
            due,
            interval,
            memorisation_factor,
        };
        let actual = RevisionSettings::new(due, interval, memorisation_factor);
        assert_eq!(expected, actual);
    }

    #[test]
    fn default() {
        let expected = RevisionSettings {
            due: Utc::now(),
            interval: 0.0,
            memorisation_factor: 1300.0,
        };
        let actual = RevisionSettings::default();
        assertions::assert_revision_settings_near(&expected, &actual, 2);
    }

    #[rstest]
    #[case::default(123.0, Utc::now() - Duration::days(123), 1.0, 2.0, 6.0, 1.0, 1.0)]
    #[case::when_days_overdue_is_fractional(0.5, Utc::now() - Duration::hours(12), 8.0, 5.0, 3.0, 1.0, 1.0)]
    #[case::when_fail_coef_is_0(0.0, Utc::now(), 1e10, 1e10, 0.0, 24.0, 1.0)]
    fn create_interval_calculation_settings(
        #[case] n_days_overdue: f64,
        #[case] due: DateTime<Utc>,
        #[case] pass_coef: f64,
        #[case] easy_coef: f64,
        #[case] fail_coef: f64,
        #[case] interval: f64,
        #[case] memorisation_factor: f64,
    ) {
        let coefficients = IntervalCoefficients::new(pass_coef, easy_coef, fail_coef);
        let expected = make_interval_calculation_settings(&coefficients, n_days_overdue);
        let revision_settings = RevisionSettings::new(due, interval, memorisation_factor);
        let actual = revision_settings.create_interval_calculation_settings(&coefficients);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::when_non_0(10.0, 240.0)]
    #[case::when_0(0.0, 0.0)]
    fn calculate_fail_interval(#[case] fail_coef: f64, #[case] expected: f64) {
        let revision_settings = RevisionSettings::new(Utc::now(), 24.0, 1.0);
        let coefficients = IntervalCoefficients::new(1e10, 1e10, fail_coef);
        let calculation_settings = make_interval_calculation_settings(&coefficients, 1.0);
        let actual = revision_settings.calculate_fail_interval(&calculation_settings);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::default(1.0, 1.0, 2.4)]
    #[case::when_interval_is_high(100.0, 0.1, 101.0)]
    #[case::when_pass_coef_is_0(1.0, 0.0, 2.0)]
    fn calculate_hard_interval(
        #[case] interval: f64,
        #[case] pass_coef: f64,
        #[case] expected: f64,
    ) {
        let revision_settings = RevisionSettings::new(Utc::now(), interval, 1.0);
        let coefficients = IntervalCoefficients::new(pass_coef, 0.1, 0.1);
        let calculation_settings = make_interval_calculation_settings(&coefficients, 4.0);
        let actual = revision_settings.calculate_hard_interval(&calculation_settings);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::default(10.0, 1000.0, 5.0, 5.0, 20.0, 100.0)]
    #[case::when_pass_coef_is_0(1.0, 1.0, 0.0, 1.0, 1.0, 2.0)]
    #[case::when_memorisation_factor_is_0(1.0, 0.0, 0.1, 1.0, 1.0, 2.0)]
    #[case::when_hard_interval_is_high(1.0, 1.0, 0.1, 1000.0, 1.0, 1001.0)]
    fn calculate_pass_interval(
        #[case] interval: f64,
        #[case] memorisation_factor: f64,
        #[case] pass_coef: f64,
        #[case] hard_interval: f64,
        #[case] days_overdue: f64,
        #[case] expected: f64,
    ) {
        let revision_settings = RevisionSettings::new(Utc::now(), interval, memorisation_factor);
        let coefficients = IntervalCoefficients::new(pass_coef, 1.3, 0.0);
        let calculation_settings = make_interval_calculation_settings(&coefficients, days_overdue);
        let actual =
            revision_settings.calculate_pass_interval(&calculation_settings, hard_interval);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::default(10.0, 2000.0, 5.0, 100.0, 4.0, 20.0, 30000.0)]
    #[case::when_pass_interval_is_high(1.0, 1.0, 0.1, 0.1, 100.0, 1.0, 101.0)]
    #[case::when_memorisation_factor_is_0(1.0, 0.0, 0.1, 0.1, 1.0, 1.0, 2.0)]
    #[case::when_pass_coef_is_0(1.0, 1.0, 0.0, 0.1, 1.0, 1.0, 2.0)]
    #[case::when_easy_coef_is_0(1.0, 1.0, 0.1, 0.0, 1.0, 1.0, 2.0)]
    fn calculate_easy_interval(
        #[case] interval: f64,
        #[case] memorisation_factor: f64,
        #[case] pass_coef: f64,
        #[case] easy_coef: f64,
        #[case] pass_interval: f64,
        #[case] days_overdue: f64,
        #[case] expected: f64,
    ) {
        let revision_settings = RevisionSettings::new(Utc::now(), interval, memorisation_factor);
        let coefficients = IntervalCoefficients::new(pass_coef, easy_coef, 0.0);
        let calculation_settings = make_interval_calculation_settings(&coefficients, days_overdue);
        let actual =
            revision_settings.calculate_easy_interval(&calculation_settings, pass_interval);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_possible_intervals() {
        let interval = 1.0;
        let factor = 2000.0;
        let days_overdue = 4.0;
        let pass_coef = 1.0;
        let easy_coef = 2.0;
        let fail_coef = 0.0;
        let due = Utc::now() - Duration::days(days_overdue as i64);
        let revision_settings = RevisionSettings::new(due, interval, factor);
        let coefficients = IntervalCoefficients::new(pass_coef, easy_coef, fail_coef);
        let expected = PossibleIntervals(0.0, 2.4, 6.0, 20.0);
        let actual = revision_settings.calculate_possible_intervals(&coefficients);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::fail_score(Score::Fail, 0.0)]
    #[case::hard_score(Score::Hard, 2.4)]
    #[case::pass_score(Score::Pass, 6.0)]
    #[case::easy_score(Score::Easy, 20.0)]
    fn calculate_interval_with_fail_score(#[case] score: Score, #[case] expected: f64) {
        let due = Utc::now() - Duration::days(4);
        let revision_settings = RevisionSettings::new(due, 1.0, 2000.0);
        let coefficients = IntervalCoefficients::new(1.0, 2.0, 0.0);
        let actual = revision_settings.calculate_new_interval(&score, &coefficients);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::when_fail_and_factor_gt_1300(Score::Fail, 2000.0, 1800.0)]
    #[case::when_fail_and_factor_lt_1300(Score::Fail, 0.0, 1300.0)]
    #[case::when_hard_and_factor_gt_1300(Score::Hard, 2000.0, 1850.0)]
    #[case::when_hard_and_factor_lt_1300(Score::Hard, 0.0, 1300.0)]
    #[case::when_pass_and_factor_gt_1300(Score::Pass, 2000.0, 2000.0)]
    #[case::when_pass_and_factor_lt_1300(Score::Pass, 0.0, 1300.0)]
    #[case::when_easy_and_factor_gt_1300(Score::Easy, 2000.0, 2150.0)]
    #[case::when_easy_and_factor_lt_1300(Score::Easy, 0.0, 1300.0)]
    fn calculate_new_memorisation_factor(
        #[case] score: Score,
        #[case] memorisation_factor: f64,
        #[case] expected: f64,
    ) {
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, memorisation_factor);
        let actual = revision_settings.calculate_new_memorisation_factor(&score);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_new_due_date() {
        let new_interval = 15.5;
        let original_due_date = Utc::now();
        let revision_settings = RevisionSettings::new(original_due_date, 0.0, 0.0);
        let expected = original_due_date + test_tools::duration_from_interval(new_interval);
        let actual = revision_settings.calculate_new_due_date(new_interval);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::when_fail(Score::Fail, 0.0, 1800.0)]
    #[case::when_hard(Score::Hard, 2.4, 1850.0)]
    #[case::when_pass(Score::Pass, 6.0, 2000.0)]
    #[case::when_easy(Score::Easy, 20.0, 2150.0)]
    fn transform(
        #[case] score: Score,
        #[case] expected_interval: f64,
        #[case] expected_memorisation_factor: f64,
    ) {
        let original_due_date = Utc::now() - Duration::days(4);
        let original_memorisation_factor = 2000.0;
        let original_interval = 1.0;
        let revision_settings = RevisionSettings::new(original_due_date, 1.0, 2000.0);
        let coefficients = IntervalCoefficients::new(1.0, 2.0, 0.0);
        let expected = test_tools::make_expected_revision_settings(
            &original_due_date,
            expected_interval,
            expected_memorisation_factor,
        );
        let actual = revision_settings.transform(score, &coefficients);
        assert_eq!(expected, actual);
    }
}
