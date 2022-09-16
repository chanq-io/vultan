use crate::application_state::deck::IntervalCoefficients;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq)]
struct IntervalCalculationSettings {
    coefficients: IntervalCoefficients,
    days_overdue: f64
}

#[derive(Clone, Debug, PartialEq)]
pub struct RevisionSettings {
    pub due: DateTime<Utc>,
    pub interval: f64,
    pub memorisation_factor: f64,
}

impl RevisionSettings {
    // fn with_calculated_interval(
    //     self,
    //     exponential_backoff_coefficients: &ExponentialBackoffCoefficients,
    //     interval: f64,
    // ) -> RevisionSettings {
    //     self
    // }
    //
    fn new(due: DateTime<Utc>, interval: f64, memorisation_factor: f64) -> Self {
        RevisionSettings {
            due,
            interval,
            memorisation_factor
        }
    }
    fn calculate_fail_interval(&self, interval_calculation_settings: &IntervalCalculationSettings) -> f64 {
        self.interval * interval_calculation_settings.coefficients.fail_coef
    }

    fn calculate_hard_interval(&self, calculation_settings: &IntervalCalculationSettings) -> f64 {
        let fallback = self.interval + 1.0;
        let base_num_days = self.interval + calculation_settings.days_overdue * 0.25;
        fallback.max(1.2 * base_num_days * calculation_settings.coefficients.pass_coef)
    }

    fn calculate_pass_interval(
        &self,
        interval_calculation_settings: &IntervalCalculationSettings,
        hard_interval: f64,
    ) -> f64 {
        let fallback = hard_interval + 1.0;
        let base_num_days = self.interval + interval_calculation_settings.days_overdue * 0.5;
        let memorisation_coef = self.memorisation_factor * 0.001;
        let pass_coef = interval_calculation_settings.coefficients.pass_coef;
        fallback.max(base_num_days * memorisation_coef * pass_coef)
    }

    fn calculate_easy_interval(
        &self,
        interval_calculation_settings: &IntervalCalculationSettings,
        pass_interval: f64,
    ) -> f64 {
        0.0
    }
}

fn calculate_days_between(present: DateTime<Utc>, past: DateTime<Utc>) -> i64 {
    if past > present {
        panic!("past > present. Possible state file corruption");
    }
    present.signed_duration_since(past).num_days()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_interval_calculation_settings(
        pass_coef: f64,
        easy_coef: f64,
        fail_coef: f64,
        days_overdue: f64
    ) -> IntervalCalculationSettings {
        IntervalCalculationSettings {
            coefficients: IntervalCoefficients {
                pass_coef,
                easy_coef,
                fail_coef,
            },
            days_overdue
        }
    }

    #[test]
    fn new_revision_settings()  {
        let due = Utc::now();
        let interval = 123.0;
        let memorisation_factor = 234.5;
        let expected = RevisionSettings {
            due,
            interval,
            memorisation_factor
        };
        let actual = RevisionSettings::new(due, interval, memorisation_factor);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_fail_interval_where_fail_coef_is_0() {
        let revision_settings = RevisionSettings::new(Utc::now(), 24.0, 1.0);
        let coefficients = make_interval_calculation_settings(1e10, 1e10, 0.0, 1.0);
        let expected = 0.0;
        let actual = revision_settings.calculate_fail_interval(&coefficients);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_fail_interval_where_fail_coef_is_non_0() {
        let revision_settings = RevisionSettings::new(Utc::now(), 24.0, 1.0);
        let coefficients = make_interval_calculation_settings(1e10, 1e10, 10.0, 1.0);
        let expected = 240.0;
        let actual = revision_settings.calculate_fail_interval(&coefficients);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_hard_interval_where_interval_is_already_high() {
        let interval = 100.0;
        let revision_settings = RevisionSettings::new(Utc::now(), interval, 1.0);
        let coefficients = make_interval_calculation_settings(0.1, 0.1, 0.1, 1.0);
        let expected = interval + 1.0;
        let actual = revision_settings.calculate_hard_interval(&coefficients);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_hard_interval_where_pass_coef_is_0() {
        let interval = 1.0;
        let pass_coef = 0.0;
        let revision_settings = RevisionSettings::new(Utc::now(), interval, 1.0);
        let coefficients = make_interval_calculation_settings(pass_coef, 0.1, 0.1, 1.0);
        let expected = interval + 1.0;
        let actual = revision_settings.calculate_hard_interval(&coefficients);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_hard_interval() {
        let interval = 1.0;
        let revision_settings = RevisionSettings::new(Utc::now(), interval, 1.0);
        let days_overdue = 4.0;
        let pass_coef = 1.0;
        let coefficients = make_interval_calculation_settings(
            pass_coef,
            1.3,
            0.0,
            days_overdue
        );
        let expected = 2.4;
        let actual = revision_settings.calculate_hard_interval(&coefficients);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_pass_interval_where_pass_coef_is_0() {
        let pass_coef = 0.0;
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, 1.0);
        let coefficients = make_interval_calculation_settings(pass_coef, 0.1, 0.1, 1.0);
        let hard_interval = 1.0;
        let expected = hard_interval + 1.0;
        let actual = revision_settings.calculate_pass_interval(&coefficients, hard_interval);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_pass_interval_where_factor_is_0() {
        let memorisation_factor = 0.0;
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, memorisation_factor);
        let coefficients = make_interval_calculation_settings(0.1, 0.1, 0.1, 1.0);
        let hard_interval = 1.0;
        let expected = hard_interval + 1.0;
        let actual = revision_settings.calculate_pass_interval(&coefficients, hard_interval);
        assert_eq!(expected, actual);
    }

    #[test]
    fn calculate_pass_interval_where_hard_interval_is_already_high() {
        let memorisation_factor = 0.0;
        let revision_settings = RevisionSettings::new(Utc::now(), 1.0, memorisation_factor);
        let coefficients = make_interval_calculation_settings(0.1, 0.1, 0.1, 1.0);
        let hard_interval = 100.0;
        let expected = hard_interval + 1.0;
        let actual = revision_settings.calculate_pass_interval(&coefficients, hard_interval);
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
        let coefficients = make_interval_calculation_settings(
            pass_coef,
            1.3,
            0.0,
            days_overdue
        );
        let expected = 100.0;
        let actual = revision_settings.calculate_pass_interval(&coefficients, hard_interval);
        assert_eq!(expected, actual);
    }

    // i4 = max(i3 + 1, (i + d) * (f / 1000.0) * m * m4)

    // #[test]
    // fn with_interval() {
    //     let revision_settings = make_default_revision_settings();
    //
    // }
}
