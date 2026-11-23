use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::{prelude::*, pyclass::CompareOp, Python};

#[pyclass]
struct OrderedDunderMethods(i64);

#[pymethods]
impl OrderedDunderMethods {
    fn __lt__(&self, other: &Self) -> bool {
        self.0 < other.0
    }

    fn __le__(&self, other: &Self) -> bool {
        self.0 <= other.0
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }

    fn __ne__(&self, other: &Self) -> bool {
        self.0 != other.0
    }

    fn __gt__(&self, other: &Self) -> bool {
        self.0 > other.0
    }

    fn __ge__(&self, other: &Self) -> bool {
        self.0 >= other.0
    }
}

#[pyclass]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct OrderedRichcmp(i64);

#[pymethods]
impl OrderedRichcmp {
    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        op.matches(self.cmp(other))
    }
}

#[pyclass]
struct AddSame(i64);

#[pymethods]
impl AddSame {
    fn __add__(&self, other: &Self) -> i64 {
        self.0 + other.0
    }
}

#[pyclass]
struct RaddFromAddSame(i64);

#[pymethods]
impl RaddFromAddSame {
    fn __radd__(&self, other: &AddSame) -> i64 {
        self.0 + other.0
    }
}

fn bench_ordered_dunder_methods(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let obj1 = &Bound::new(py, OrderedDunderMethods(0)).unwrap().into_any();
        let obj2 = &Bound::new(py, OrderedDunderMethods(1)).unwrap().into_any();

        b.iter(|| obj2.gt(obj1).unwrap());
    });
}

fn bench_ordered_richcmp(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let obj1 = &Bound::new(py, OrderedRichcmp(0)).unwrap().into_any();
        let obj2 = &Bound::new(py, OrderedRichcmp(1)).unwrap().into_any();

        b.iter(|| obj2.gt(obj1).unwrap());
    });
}

fn bench_dunder_eq_mismatched_type(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let obj = &Bound::new(py, OrderedDunderMethods(0)).unwrap().into_any();
        let none = py.None().into_bound(py);

        // `__eq__` rejects `None` with NotImplemented, then CPython falls back to identity.
        b.iter(|| obj.eq(&none).unwrap());
    });
}

fn bench_richcmp_eq_mismatched_type(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let obj = &Bound::new(py, OrderedRichcmp(0)).unwrap().into_any();
        let none = py.None().into_bound(py);

        // `__richcmp__` rejects `None` with NotImplemented, then CPython falls back to identity.
        b.iter(|| obj.eq(&none).unwrap());
    });
}

fn bench_add_mismatched_type_reflected(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let lhs = &Bound::new(py, AddSame(1)).unwrap().into_any();
        let rhs = &Bound::new(py, RaddFromAddSame(2)).unwrap().into_any();

        // `AddSame.__add__` rejects the operand with NotImplemented, then
        // `RaddFromAddSame.__radd__` handles it.
        b.iter(|| lhs.add(rhs).unwrap());
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("ordered_dunder_methods", bench_ordered_dunder_methods);
    c.bench_function("ordered_richcmp", bench_ordered_richcmp);
    c.bench_function(
        "dunder_eq_mismatched_type",
        bench_dunder_eq_mismatched_type,
    );
    c.bench_function(
        "richcmp_eq_mismatched_type",
        bench_richcmp_eq_mismatched_type,
    );
    c.bench_function(
        "add_mismatched_type_reflected",
        bench_add_mismatched_type_reflected,
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
