use std::hint::black_box;

use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};
use rust_decimal::Decimal;

use pyo3::prelude::*;
use pyo3::types::PyDict;

fn decimal_via_extract(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let locals = PyDict::new(py);
        py.run(
            cr#"
import decimal
py_dec = decimal.Decimal("0.0")
"#,
            None,
            Some(&locals),
        )
        .unwrap();
        let py_dec = locals.get_item("py_dec").unwrap().unwrap();

        b.iter(|| black_box(&py_dec).extract::<Decimal>().unwrap());
    })
}

fn decimal_via_extract_nontrivial(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let py_dec = py
            .import("decimal")
            .unwrap()
            .getattr("Decimal")
            .unwrap()
            .call1(("12345.6789",))
            .unwrap();

        b.iter(|| black_box(&py_dec).extract::<Decimal>().unwrap());
    })
}

fn decimal_via_extract_int(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let py_int = 12345i64.into_pyobject(py).unwrap();

        b.iter(|| black_box(&py_int).extract::<Decimal>().unwrap());
    })
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("decimal_via_extract", decimal_via_extract);
    c.bench_function(
        "decimal_via_extract_nontrivial",
        decimal_via_extract_nontrivial,
    );
    c.bench_function("decimal_via_extract_int", decimal_via_extract_int);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
