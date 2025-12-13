//! Custom benchmarking for general functions.
pub use std::hint::black_box;
use std::{
    collections::BTreeMap,
    fmt::Display,
    ops::{Add, Div, Mul},
    time::{Duration, Instant},
};

use super::writer::CsvEntry;

const NANOSECOND_IN_NANOS: u128 = 1;
const MICROSECOND_IN_NANOS: u128 = 1_000 * NANOSECOND_IN_NANOS;
const MILLISECOND_IN_NANOS: u128 = 1_000 * MICROSECOND_IN_NANOS;
const SECOND_IN_NANOS: u128 = 1_000 * MILLISECOND_IN_NANOS;
const MINUTE_IN_NANOS: u128 = 60 * SECOND_IN_NANOS;

#[derive(Clone, Debug)]
pub struct BenchmarkResult {
    pub name: String,
    pub time_limit: Duration,
    pub iterations: u128,
    pub fastest: Duration,
    pub slowest: Duration,
    pub mean: Duration,
    pub std_dev: Duration,
    pub median: Duration,
    pub mad: Duration,
}

impl BenchmarkResult {
    pub fn human_readable_format(&self) -> impl Fn(Duration) -> String {
        // Majority voting of scale to use the most readable output.
        let scales = [
            (MINUTE_IN_NANOS, "m"),
            (SECOND_IN_NANOS, "s"),
            (MILLISECOND_IN_NANOS, "ms"),
            (MICROSECOND_IN_NANOS, "µs"),
            (NANOSECOND_IN_NANOS, "ns"),
        ];
        let (scale, unit) = [
            self.fastest,
            self.slowest,
            self.mean,
            self.std_dev,
            self.median,
            self.mad,
        ]
        .iter()
        .map(|d| {
            scales
                .iter()
                .find(|(scale, _)| d.as_nanos() >= *scale)
                .unwrap_or(&scales[0])
        })
        .fold(BTreeMap::new(), |mut acc, key| {
            acc.entry(key).and_modify(|cnt| *cnt += 1).or_insert(1);
            acc
        })
        .into_iter()
        .max_by_key(|(_, cnt)| *cnt)
        .map_or(scales[0], |(&(scale, unit), _)| (scale, unit));
        // The f64 uses 53 bits of precision, which is already large enough to hold more
        // than 100 days in nanoseconds, which should be more than enough for any
        // benchmark.
        #[allow(clippy::cast_precision_loss)]
        {
            let scale = scale as f64;
            move |d: Duration| format!("{:.3}{unit}", d.as_nanos() as f64 / scale)
        }
    }
}

impl Display for BenchmarkResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatter = self.human_readable_format();
        write!(
            f,
            "[{}] fastest: {}, slowest: {}, mean: {}, std_dev: {}, median: {}, mad: {} | {} iterations in {:?}",
            self.name,
            formatter(self.fastest),
            formatter(self.slowest),
            formatter(self.mean),
            formatter(self.std_dev),
            formatter(self.median),
            formatter(self.mad),
            self.iterations,
            self.time_limit,
        )
    }
}

impl CsvEntry for BenchmarkResult {
    fn columns() -> Vec<String> {
        vec![
            "name".to_owned(),
            "iterations".to_owned(),
            "time_limit".to_owned(),
            "fastest".to_owned(),
            "slowest".to_owned(),
            "mean".to_owned(),
            "std_dev".to_owned(),
            "median".to_owned(),
            "mad".to_owned(),
        ]
    }

    fn values(&self) -> Vec<String> {
        let formatter = self.human_readable_format();
        vec![
            self.name.clone(),
            self.iterations.to_string(),
            format!("{:?}", self.time_limit),
            formatter(self.fastest),
            formatter(self.slowest),
            formatter(self.mean),
            formatter(self.std_dev),
            formatter(self.median),
            formatter(self.mad),
        ]
    }
}

/// A simple square root function using Newton's method.
fn sqrt<T>(x: T) -> T
where
    T: PartialOrd + Copy + From<u8> + Add<Output = T> + Mul<Output = T> + Div<Output = T>,
{
    let mut y = (x + T::from(1)) / T::from(2);
    while y * y > x {
        y = (y + x / y) / T::from(2);
    }
    y
}

fn med<T>(mut v: Vec<T>) -> T
where
    T: Ord + Copy + From<u8> + Add<Output = T> + Div<Output = T>,
{
    let length = v.len();
    assert!(length > 0, "Cannot compute median of an empty list");
    if length.is_multiple_of(2) {
        let (_, &mut med_left, right) = v.select_nth_unstable(length / 2);
        let (_, &mut med_right, _) = right.select_nth_unstable(0);
        (med_left + med_right) / T::from(2)
    } else {
        let (_, &mut median, _) = v.select_nth_unstable(length / 2);
        median
    }
}

pub fn measure_once<F, T>(f: F) -> Duration
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let _ = black_box(f());
    let end = Instant::now();
    end.duration_since(start)
}

pub fn measure_many<F, T, S>(name: S, time_limit: Duration, mut f: F) -> BenchmarkResult
where
    F: FnMut() -> T,
    S: AsRef<str>,
{
    // Cold run to get a sense of how long a single run takes, which will be used to
    // determine how many iterations we can run in the given time limit.
    let single_run = measure_once(&mut f);
    let iterations = time_limit.as_nanos() / single_run.as_nanos();
    // Get 1% or u32::MAX of iterations as burn-in iterations to avoid cold run
    // issues, and also provide a better estimate of the time limit.
    #[allow(clippy::cast_possible_truncation)]
    let burn_in = (iterations / 100).max(1).min(u128::from(u32::MAX)) as u32;
    let cold_run_time = (0..burn_in).map(|_| measure_once(&mut f)).sum::<Duration>() / burn_in;
    // Update the estimation of iterations to account for burn-in.
    let iterations = time_limit.as_nanos() / cold_run_time.as_nanos();
    let iterations = match iterations {
        ..10 => iterations.min(3),
        10..100 => iterations / 10 * 10,
        100..1000 => iterations / 100 * 100,
        _ => (iterations / 1000 * 1000).min(1_000_000),
    };
    let measurements = (0..iterations)
        .map(|_| black_box(measure_once(&mut f)).as_nanos())
        .collect::<Vec<_>>();
    let unreachable_by_multi_test = || unreachable!("At least 3 measurements should be taken");
    let &fastest = measurements
        .iter()
        .min()
        .unwrap_or_else(unreachable_by_multi_test);
    let &slowest = measurements
        .iter()
        .max()
        .unwrap_or_else(unreachable_by_multi_test);
    let mean = measurements.iter().sum::<u128>() / iterations;
    let std_dev = sqrt(
        measurements
            .iter()
            .map(|&x| x.abs_diff(mean).pow(2))
            .sum::<u128>()
            / iterations,
    );
    let median = med(measurements.clone());
    let mad = med(measurements
        .iter()
        .map(|&x| x.abs_diff(median))
        .collect::<Vec<_>>());
    // We allow the cast here, because even u64 is large enough to hold values that
    // are over 500 years in nanoseconds. No test results will ever be that large.
    #[allow(clippy::cast_possible_truncation)]
    BenchmarkResult {
        name: name.as_ref().to_owned(),
        time_limit,
        iterations,
        fastest: Duration::from_nanos(fastest as u64),
        slowest: Duration::from_nanos(slowest as u64),
        mean: Duration::from_nanos(mean as u64),
        std_dev: Duration::from_nanos(std_dev as u64),
        median: Duration::from_nanos(median as u64),
        mad: Duration::from_nanos(mad as u64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqrt() {
        for i in 0..=100 {
            let sqrt_i = sqrt(i);
            assert!(sqrt_i * sqrt_i <= i);
            assert!((sqrt_i + 1) * (sqrt_i + 1) > i);
        }
    }
}
