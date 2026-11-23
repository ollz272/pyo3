use std::hint::black_box;

use codspeed_criterion_compat::{Bencher, Criterion, criterion_group, criterion_main};

use pyo3::prelude::*;
use pyo3::types::{PyDate, PyDateAccess, PyDateTime, PyDelta, PyDeltaAccess, PyTime, PyTimeAccess};

fn date_fields(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let date = PyDate::new(py, 2020, 6, 15).unwrap();
        bench.iter(|| {
            let d = black_box(&date);
            (d.get_year(), d.get_month(), d.get_day())
        });
    });
}

fn datetime_fields(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let datetime = PyDateTime::new(py, 2020, 6, 15, 12, 30, 45, 123_456, None).unwrap();
        bench.iter(|| {
            let dt = black_box(&datetime);
            (
                dt.get_year(),
                dt.get_month(),
                dt.get_day(),
                dt.get_hour(),
                dt.get_minute(),
                dt.get_second(),
                dt.get_microsecond(),
            )
        });
    });
}

fn time_fields(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let time = PyTime::new(py, 12, 30, 45, 123_456, None).unwrap();
        bench.iter(|| {
            let t = black_box(&time);
            (
                t.get_hour(),
                t.get_minute(),
                t.get_second(),
                t.get_microsecond(),
            )
        });
    });
}

fn delta_fields(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let delta = PyDelta::new(py, 1, 3_600, 123_456, false).unwrap();
        bench.iter(|| {
            let d = black_box(&delta);
            (d.get_days(), d.get_seconds(), d.get_microseconds())
        });
    });
}

fn extract_chrono_date(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let date = PyDate::new(py, 2020, 6, 15).unwrap().into_any();
        bench.iter(|| black_box(&date).extract::<chrono::NaiveDate>().unwrap());
    });
}

fn extract_chrono_datetime(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let datetime = PyDateTime::new(py, 2020, 6, 15, 12, 30, 45, 123_456, None)
            .unwrap()
            .into_any();
        bench.iter(|| {
            black_box(&datetime)
                .extract::<chrono::NaiveDateTime>()
                .unwrap()
        });
    });
}

fn extract_chrono_time(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let time = PyTime::new(py, 12, 30, 45, 123_456, None)
            .unwrap()
            .into_any();
        bench.iter(|| black_box(&time).extract::<chrono::NaiveTime>().unwrap());
    });
}

fn extract_chrono_timedelta(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let delta = PyDelta::new(py, 1, 3_600, 123_456, false)
            .unwrap()
            .into_any();
        bench.iter(|| black_box(&delta).extract::<chrono::TimeDelta>().unwrap());
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("date_fields", date_fields);
    c.bench_function("datetime_fields", datetime_fields);
    c.bench_function("time_fields", time_fields);
    c.bench_function("delta_fields", delta_fields);
    c.bench_function("extract_chrono_date", extract_chrono_date);
    c.bench_function("extract_chrono_datetime", extract_chrono_datetime);
    c.bench_function("extract_chrono_time", extract_chrono_time);
    c.bench_function("extract_chrono_timedelta", extract_chrono_timedelta);
use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::prelude::*;
use pyo3::types::PyDict;

fn py_datetimes(py: Python<'_>) -> (Bound<'_, PyAny>, Bound<'_, PyAny>) {
    let locals = PyDict::new(py);
    py.run(
        cr#"
import datetime
dt_utc = datetime.datetime(2024, 1, 1, 12, 30, 45, 123456, tzinfo=datetime.timezone.utc)
dt_fixed = datetime.datetime(2024, 1, 1, 12, 30, 45, 123456, tzinfo=datetime.timezone(datetime.timedelta(hours=1)))
"#,
        None,
        Some(&locals),
    )
    .unwrap();
    (
        locals.get_item("dt_utc").unwrap().unwrap(),
        locals.get_item("dt_fixed").unwrap().unwrap(),
    )
}

fn chrono_fixed_offset_into_pyobject(b: &mut Bencher<'_>, offset_secs: i32) {
    Python::attach(|py| {
        let dt = chrono::DateTime::from_timestamp(1_704_112_245, 123_456_000)
            .unwrap()
            .with_timezone(&chrono::FixedOffset::east_opt(offset_secs).unwrap());
        b.iter(|| black_box(dt).into_pyobject(py).unwrap());
    })
}

fn chrono_fixed_offset_utc_into_pyobject(b: &mut Bencher<'_>) {
    chrono_fixed_offset_into_pyobject(b, 0);
}

fn chrono_fixed_offset_nonzero_into_pyobject(b: &mut Bencher<'_>) {
    chrono_fixed_offset_into_pyobject(b, 3600);
}

fn time_offset_datetime_into_pyobject(b: &mut Bencher<'_>, offset_secs: i32) {
    Python::attach(|py| {
        let dt = time::OffsetDateTime::from_unix_timestamp_nanos(1_704_112_245_123_456_000)
            .unwrap()
            .to_offset(time::UtcOffset::from_whole_seconds(offset_secs).unwrap());
        b.iter(|| black_box(dt).into_pyobject(py).unwrap());
    })
}

fn time_offset_datetime_utc_into_pyobject(b: &mut Bencher<'_>) {
    time_offset_datetime_into_pyobject(b, 0);
}

fn time_offset_datetime_nonzero_into_pyobject(b: &mut Bencher<'_>) {
    time_offset_datetime_into_pyobject(b, 3600);
}

fn extract_bench<T>(b: &mut Bencher<'_>, utc: bool)
where
    T: for<'a, 'py> FromPyObject<'a, 'py>,
    for<'a, 'py> <T as FromPyObject<'a, 'py>>::Error: std::fmt::Debug,
{
    Python::attach(|py| {
        let (dt_utc, dt_fixed) = py_datetimes(py);
        let dt = if utc { dt_utc } else { dt_fixed };
        b.iter(|| black_box(&dt).extract::<T>().unwrap());
    })
}

fn chrono_fixed_offset_utc_extract(b: &mut Bencher<'_>) {
    extract_bench::<chrono::DateTime<chrono::FixedOffset>>(b, true);
}

fn chrono_fixed_offset_nonzero_extract(b: &mut Bencher<'_>) {
    extract_bench::<chrono::DateTime<chrono::FixedOffset>>(b, false);
}

fn time_offset_datetime_utc_extract(b: &mut Bencher<'_>) {
    extract_bench::<time::OffsetDateTime>(b, true);
}

fn time_offset_datetime_nonzero_extract(b: &mut Bencher<'_>) {
    extract_bench::<time::OffsetDateTime>(b, false);
}

fn jiff_zoned_utc_extract(b: &mut Bencher<'_>) {
    extract_bench::<jiff::Zoned>(b, true);
}

fn jiff_zoned_nonzero_extract(b: &mut Bencher<'_>) {
    extract_bench::<jiff::Zoned>(b, false);
}

fn jiff_timestamp_utc_extract(b: &mut Bencher<'_>) {
    extract_bench::<jiff::Timestamp>(b, true);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function(
        "chrono_fixed_offset_utc_into_pyobject",
        chrono_fixed_offset_utc_into_pyobject,
    );
    c.bench_function(
        "chrono_fixed_offset_nonzero_into_pyobject",
        chrono_fixed_offset_nonzero_into_pyobject,
    );
    c.bench_function(
        "time_offset_datetime_utc_into_pyobject",
        time_offset_datetime_utc_into_pyobject,
    );
    c.bench_function(
        "time_offset_datetime_nonzero_into_pyobject",
        time_offset_datetime_nonzero_into_pyobject,
    );
    c.bench_function(
        "chrono_fixed_offset_utc_extract",
        chrono_fixed_offset_utc_extract,
    );
    c.bench_function(
        "chrono_fixed_offset_nonzero_extract",
        chrono_fixed_offset_nonzero_extract,
    );
    c.bench_function(
        "time_offset_datetime_utc_extract",
        time_offset_datetime_utc_extract,
    );
    c.bench_function(
        "time_offset_datetime_nonzero_extract",
        time_offset_datetime_nonzero_extract,
    );
    c.bench_function("jiff_zoned_utc_extract", jiff_zoned_utc_extract);
    c.bench_function("jiff_zoned_nonzero_extract", jiff_zoned_nonzero_extract);
    c.bench_function("jiff_timestamp_utc_extract", jiff_timestamp_utc_extract);
}

use pyo3::types::{PyDateTime, PyTime};

fn datetime_new_with_fold_false(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        b.iter(|| {
            PyDateTime::new_with_fold(py, 2024, 6, 15, 12, 30, 45, 123456, None, black_box(false))
                .unwrap()
        });
    })
}

fn datetime_new_with_fold_true(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        b.iter(|| {
            PyDateTime::new_with_fold(py, 2024, 6, 15, 12, 30, 45, 123456, None, black_box(true))
                .unwrap()
        });
    })
}

fn time_new_with_fold_false(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        b.iter(|| PyTime::new_with_fold(py, 12, 30, 45, 123456, None, black_box(false)).unwrap());
    })
}

fn time_new_with_fold_true(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        b.iter(|| PyTime::new_with_fold(py, 12, 30, 45, 123456, None, black_box(true)).unwrap());
    })
}

fn fold_benchmark(c: &mut Criterion) {
    c.bench_function("datetime_new_with_fold_false", datetime_new_with_fold_false);
    c.bench_function("datetime_new_with_fold_true", datetime_new_with_fold_true);
    c.bench_function("time_new_with_fold_false", time_new_with_fold_false);
    c.bench_function("time_new_with_fold_true", time_new_with_fold_true);
}

criterion_group!(benches, criterion_benchmark, fold_benchmark);
criterion_main!(benches);
