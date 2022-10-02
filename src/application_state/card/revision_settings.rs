use super::score::Score;
use crate::application_state::deck::IntervalCoefficients;
use chrono::{DateTime, Duration, Utc};

#[derive(Clone, Debug, PartialEq)]
pub struct RevisionSettings {
    pub due: DateTime<Utc>,
    pub interval: f64,
    pub memorisation_factor: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct IntervalCalculationSettings<'a> {
    coefficients: &'a IntervalCoefficients,
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

    pub fn default() -> Self {
        Self {
            due: Utc::now(),
            interval: 0.0,
            memorisation_factor: 1300.0,
        }
    }

    pub fn transform(self, score: Score, coefficients: &IntervalCoefficients) -> Self {
        let new_interval = self.calculate_new_interval(&score, &coefficients);
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
        IntervalCalculationSettings {
            coefficients,
            days_overdue: present.signed_duration_since(past).num_days() as f64,
            // TODO: this should probably be an accurate float otherwise we're discarding info
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

#[cfg(test)]
pub mod assertions {
    use super::*;
    pub fn assert_near(
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

#[cfg(test)]
mod unit_tests {
    use super::*;
    use chrono::Duration;

    fn make_interval_calculation_settings<'a>(
        coefficients: &'a IntervalCoefficients,
        days_overdue: f64,
    ) -> IntervalCalculationSettings<'a> {
        IntervalCalculationSettings {
            coefficients,
            days_overdue,
        }
    }

    fn duration_from_interval(interval: f64) -> Duration {
        Duration::seconds((86400.0 * interval) as i64)
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
        assertions::assert_near(&expected, &actual, 2);
    }

    #[test]
    fn create_interval_calculation_settings() {
        let n_days_overdue = 123.0;
        let due = Utc::now() - Duration::days(n_days_overdue as i64);
        let pass_coef = 1.0;
        let easy_coef = 2.0;
        let fail_coef = 3.0;
        let coefficients = IntervalCoefficients::new(pass_coef, easy_coef, fail_coef);
        let expected = make_interval_calculation_settings(&coefficients, n_days_overdue);
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, 1.0);
        let actual = revision_settings.create_interval_calculation_settings(&coefficients);
    }

    #[ignore]
    #[test]
    fn create_interval_calculation_settings_when_days_overdue_is_fractional() {
        assert!(false);
    }

    #[test]
    fn calculate_fail_interval_where_fail_coef_is_0() {
        let revision_settings = RevisionSettings::new(Utc::now(), 24.0, 1.0);
        let coefficients = IntervalCoefficients::new(1e10, 1e10, 0.0);
        let calculation_settings = make_interval_calculation_settings(&coefficients, 1.0);
        let expected = 0.0;
        let actual = revision_settings.calculate_fail_interval(&calculation_settings);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_fail_interval_where_fail_coef_is_non_0() {
        let revision_settings = RevisionSettings::new(Utc::now(), 24.0, 1.0);
        let coefficients = IntervalCoefficients::new(1e10, 1e10, 10.0);
        let calculation_settings = make_interval_calculation_settings(&coefficients, 1.0);
        let expected = 240.0;
        let actual = revision_settings.calculate_fail_interval(&calculation_settings);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_hard_interval_where_interval_is_already_high() {
        let interval = 100.0;
        let revision_settings = RevisionSettings::new(Utc::now(), interval, 1.0);
        let coefficients = IntervalCoefficients::new(0.1, 0.1, 0.1);
        let calculation_settings = make_interval_calculation_settings(&coefficients, 1.0);
        let expected = interval + 1.0;
        let actual = revision_settings.calculate_hard_interval(&calculation_settings);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_hard_interval_where_pass_coef_is_0() {
        let interval = 1.0;
        let pass_coef = 0.0;
        let revision_settings = RevisionSettings::new(Utc::now(), interval, 1.0);
        let coefficients = IntervalCoefficients::new(pass_coef, 0.1, 0.1);
        let calculation_settings = make_interval_calculation_settings(&coefficients, 1.0);
        let expected = interval + 1.0;
        let actual = revision_settings.calculate_hard_interval(&calculation_settings);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_hard_interval() {
        let interval = 1.0;
        let revision_settings = RevisionSettings::new(Utc::now(), interval, 1.0);
        let days_overdue = 4.0;
        let pass_coef = 1.0;
        let coefficients = IntervalCoefficients::new(pass_coef, 0.1, 0.1);
        let calculation_settings = make_interval_calculation_settings(&coefficients, days_overdue);
        let expected = 2.4;
        let actual = revision_settings.calculate_hard_interval(&calculation_settings);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_pass_interval_where_pass_coef_is_0() {
        let pass_coef = 0.0;
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, 1.0);
        let coefficients = IntervalCoefficients::new(pass_coef, 0.1, 0.1);
        let calculation_settings = make_interval_calculation_settings(&coefficients, 1.0);
        let hard_interval = 1.0;
        let expected = hard_interval + 1.0;
        let actual =
            revision_settings.calculate_pass_interval(&calculation_settings, hard_interval);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_pass_interval_where_factor_is_0() {
        let memorisation_factor = 0.0;
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, memorisation_factor);
        let coefficients = IntervalCoefficients::new(0.1, 0.1, 0.1);
        let calculation_settings = make_interval_calculation_settings(&coefficients, 1.0);
        let hard_interval = 1.0;
        let expected = hard_interval + 1.0;
        let actual =
            revision_settings.calculate_pass_interval(&calculation_settings, hard_interval);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_pass_interval_where_hard_interval_is_already_high() {
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, 1.0);
        let coefficients = IntervalCoefficients::new(0.1, 0.1, 0.1);
        let calculation_settings = make_interval_calculation_settings(&coefficients, 1.0);
        let hard_interval = 100.0;
        let expected = hard_interval + 1.0;
        let actual =
            revision_settings.calculate_pass_interval(&calculation_settings, hard_interval);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_pass_interval() {
        let interval = 10.0;
        let factor = 1000.0;
        let revision_settings = RevisionSettings::new(Utc::now(), interval, factor);
        let days_overdue = 20.0;
        let pass_coef = 5.0;
        let hard_interval = 5.0;
        let coefficients = IntervalCoefficients::new(pass_coef, 1.3, 0.0);
        let calculation_settings = make_interval_calculation_settings(&coefficients, days_overdue);
        let expected = 100.0;
        let actual =
            revision_settings.calculate_pass_interval(&calculation_settings, hard_interval);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_easy_interval_when_pass_interval_is_already_high() {
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, 1.0);
        let coefficients = IntervalCoefficients::new(0.1, 0.1, 0.1);
        let calculation_settings = make_interval_calculation_settings(&coefficients, 1.0);
        let pass_interval = 100.0;
        let expected = pass_interval + 1.0;
        let actual =
            revision_settings.calculate_easy_interval(&calculation_settings, pass_interval);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_easy_interval_when_factor_is_0() {
        let memorisation_factor = 0.0;
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, memorisation_factor);
        let coefficients = IntervalCoefficients::new(0.1, 0.1, 0.1);
        let calculation_settings = make_interval_calculation_settings(&coefficients, 1.0);
        let pass_interval = 1.0;
        let expected = pass_interval + 1.0;
        let actual =
            revision_settings.calculate_easy_interval(&calculation_settings, pass_interval);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_easy_interval_when_pass_coef_is_0() {
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, 1.0);
        let pass_coef = 0.0;
        let coefficients = IntervalCoefficients::new(pass_coef, 0.1, 0.1);
        let calculation_settings = make_interval_calculation_settings(&coefficients, 1.0);
        let pass_interval = 1.0;
        let expected = pass_interval + 1.0;
        let actual =
            revision_settings.calculate_easy_interval(&calculation_settings, pass_interval);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_easy_interval_when_easy_coef_is_0() {
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, 1.0);
        let easy_coef = 0.0;
        let coefficients = IntervalCoefficients::new(0.1, easy_coef, 0.1);
        let calculation_settings = make_interval_calculation_settings(&coefficients, 1.0);
        let pass_interval = 1.0;
        let expected = pass_interval + 1.0;
        let actual =
            revision_settings.calculate_easy_interval(&calculation_settings, pass_interval);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_easy_interval() {
        let interval = 10.0;
        let factor = 2000.0;
        let revision_settings = RevisionSettings::new(Utc::now(), interval, factor);
        let days_overdue = 20.0;
        let pass_coef = 5.0;
        let easy_coef = 100.0;
        let pass_interval = 4.0;
        let coefficients = IntervalCoefficients::new(pass_coef, easy_coef, 0.0);
        let calculation_settings = make_interval_calculation_settings(&coefficients, days_overdue);
        let expected = 30000.0;
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

    #[test]
    fn calculate_interval_with_fail_score() {
        let interval = 1.0;
        let factor = 2000.0;
        let days_overdue = 4.0;
        let pass_coef = 1.0;
        let easy_coef = 2.0;
        let fail_coef = 0.0;
        let score = Score::Fail;
        let due = Utc::now() - Duration::days(days_overdue as i64);
        let revision_settings = RevisionSettings::new(due, interval, factor);
        let coefficients = IntervalCoefficients::new(pass_coef, easy_coef, fail_coef);
        let expected = 0.0;
        let actual = revision_settings.calculate_new_interval(&score, &coefficients);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_interval_with_hard_score() {
        let interval = 1.0;
        let factor = 2000.0;
        let days_overdue = 4.0;
        let pass_coef = 1.0;
        let easy_coef = 2.0;
        let fail_coef = 0.0;
        let score = Score::Hard;
        let due = Utc::now() - Duration::days(days_overdue as i64);
        let revision_settings = RevisionSettings::new(due, interval, factor);
        let coefficients = IntervalCoefficients::new(pass_coef, easy_coef, fail_coef);
        let expected = 2.4;
        let actual = revision_settings.calculate_new_interval(&score, &coefficients);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_interval_with_pass_score() {
        let interval = 1.0;
        let factor = 2000.0;
        let days_overdue = 4.0;
        let pass_coef = 1.0;
        let easy_coef = 2.0;
        let fail_coef = 0.0;
        let score = Score::Pass;
        let due = Utc::now() - Duration::days(days_overdue as i64);
        let revision_settings = RevisionSettings::new(due, interval, factor);
        let coefficients = IntervalCoefficients::new(pass_coef, easy_coef, fail_coef);
        let expected = 6.0;
        let actual = revision_settings.calculate_new_interval(&score, &coefficients);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_interval_with_easy_score() {
        let interval = 1.0;
        let factor = 2000.0;
        let days_overdue = 4.0;
        let pass_coef = 1.0;
        let easy_coef = 2.0;
        let fail_coef = 0.0;
        let score = Score::Easy;
        let due = Utc::now() - Duration::days(days_overdue as i64);
        let revision_settings = RevisionSettings::new(due, interval, factor);
        let coefficients = IntervalCoefficients::new(pass_coef, easy_coef, fail_coef);
        let expected = 20.0;
        let actual = revision_settings.calculate_new_interval(&score, &coefficients);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_new_memorisation_factor_when_fail_and_factor_gt_1300() {
        let score = Score::Fail;
        let factor = 2000.0;
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, factor);
        let expected = 1800.0;
        let actual = revision_settings.calculate_new_memorisation_factor(&score);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_new_memorisation_factor_when_fail_and_factor_lt_1300() {
        let score = Score::Fail;
        let factor = 0.0;
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, factor);
        let expected = 1300.0;
        let actual = revision_settings.calculate_new_memorisation_factor(&score);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_new_memorisation_factor_when_hard_and_factor_gt_1300() {
        let score = Score::Hard;
        let factor = 2000.0;
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, factor);
        let expected = 1850.0;
        let actual = revision_settings.calculate_new_memorisation_factor(&score);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_new_memorisation_factor_when_hard_and_factor_lt_1300() {
        let score = Score::Hard;
        let factor = 0.0;
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, factor);
        let expected = 1300.0;
        let actual = revision_settings.calculate_new_memorisation_factor(&score);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_new_memorisation_factor_when_pass_and_factor_gt_1300() {
        let score = Score::Pass;
        let factor = 2000.0;
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, factor);
        let expected = 2000.0;
        let actual = revision_settings.calculate_new_memorisation_factor(&score);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_new_memorisation_factor_when_pass_and_factor_lt_1300() {
        let score = Score::Pass;
        let factor = 0.0;
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, factor);
        let expected = 1300.0;
        let actual = revision_settings.calculate_new_memorisation_factor(&score);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_new_memorisation_factor_when_easy_and_factor_gt_1300() {
        let score = Score::Easy;
        let factor = 2000.0;
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, factor);
        let expected = 2150.0;
        let actual = revision_settings.calculate_new_memorisation_factor(&score);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_new_memorisation_factor_when_easy_and_factor_lt_1300() {
        let score = Score::Easy;
        let factor = 0.0;
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, factor);
        let expected = 1300.0;
        let actual = revision_settings.calculate_new_memorisation_factor(&score);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_new_due_date() {
        let new_interval = 15.5;
        let original_due_date = Utc::now();
        let revision_settings = RevisionSettings::new(original_due_date, 0.0, 0.0);
        let expected = original_due_date + duration_from_interval(new_interval);
        let actual = revision_settings.calculate_new_due_date(new_interval);
        assert_eq!(expected, actual);
    }

    #[test]
    fn transform_when_fail() {
        let score = Score::Fail;
        let original_due_date = Utc::now() - Duration::days(4);
        let original_memorisation_factor = 2000.0;
        let original_interval = 1.0;
        let revision_settings = RevisionSettings::new(
            original_due_date,
            original_interval,
            original_memorisation_factor,
        );
        let coefficients = IntervalCoefficients::new(1.0, 2.0, 0.0);
        let expected_memorisation_factor = 1800.0;
        let expected_interval = 0.0;
        let expected_due_date = original_due_date.clone();
        let expected = RevisionSettings::new(
            expected_due_date,
            expected_interval,
            expected_memorisation_factor,
        );
        let actual = revision_settings.transform(score, &coefficients);
        assert_eq!(expected, actual);
    }

    #[test]
    fn transform_when_hard() {
        let score = Score::Hard;
        let original_due_date = Utc::now() - Duration::days(4);
        let original_memorisation_factor = 2000.0;
        let original_interval = 1.0;
        let revision_settings = RevisionSettings::new(
            original_due_date,
            original_interval,
            original_memorisation_factor,
        );
        let coefficients = IntervalCoefficients::new(1.0, 2.0, 0.0);
        let expected_interval = 2.4;
        let expected_memorisation_factor = 1850.0;
        let expected_due_date = original_due_date + duration_from_interval(expected_interval);
        let expected = RevisionSettings::new(
            expected_due_date,
            expected_interval,
            expected_memorisation_factor,
        );
        let actual = revision_settings.transform(score, &coefficients);
        assert_eq!(expected, actual);
    }

    #[test]
    fn transform_when_pass() {
        let score = Score::Pass;
        let original_due_date = Utc::now() - Duration::days(4);
        let original_memorisation_factor = 2000.0;
        let original_interval = 1.0;
        let revision_settings = RevisionSettings::new(
            original_due_date,
            original_interval,
            original_memorisation_factor,
        );
        let coefficients = IntervalCoefficients::new(1.0, 2.0, 0.0);
        let expected_interval = 6.0;
        let expected_memorisation_factor = original_memorisation_factor;
        let expected_due_date = original_due_date + duration_from_interval(expected_interval);
        let expected = RevisionSettings::new(
            expected_due_date,
            expected_interval,
            expected_memorisation_factor,
        );
        let actual = revision_settings.transform(score, &coefficients);
        assert_eq!(expected, actual);
    }

    #[test]
    fn transform_when_easy() {
        let score = Score::Easy;
        let original_due_date = Utc::now() - Duration::days(4);
        let original_memorisation_factor = 2000.0;
        let original_interval = 1.0;
        let revision_settings = RevisionSettings::new(
            original_due_date,
            original_interval,
            original_memorisation_factor,
        );
        let coefficients = IntervalCoefficients::new(1.0, 2.0, 0.0);
        let expected_interval = 20.0;
        let expected_memorisation_factor = 2150.0;
        let expected_due_date = original_due_date + duration_from_interval(expected_interval);
        let expected = RevisionSettings::new(
            expected_due_date,
            expected_interval,
            expected_memorisation_factor,
        );
        let actual = revision_settings.transform(score, &coefficients);
        assert_eq!(expected, actual);
    }
}
