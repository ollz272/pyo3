use std::hint::black_box;

use chrono::{DateTime, Utc};
use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::prelude::*;
use pyo3::types::{PyDateTime, PyTzInfo};

fn extract_utc(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let utc = PyTzInfo::utc(py).unwrap().to_owned().into_any();

        b.iter(|| black_box(&utc).extract::<Utc>().unwrap());
    })
}

fn extract_datetime_utc(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let utc = PyTzInfo::utc(py).unwrap();
        let dt = PyDateTime::new(py, 2026, 11, 20, 12, 30, 15, 123, Some(&utc))
            .unwrap()
            .into_any();

        b.iter(|| black_box(&dt).extract::<DateTime<Utc>>().unwrap());
    })
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("extract_utc", extract_utc);
    c.bench_function("extract_datetime_utc", extract_datetime_utc);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
