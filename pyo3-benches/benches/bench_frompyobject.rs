use std::hint::black_box;

use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::{
    prelude::*,
    types::{PyByteArray, PyBytes, PyList, PyString, PyTuple},
};

#[derive(FromPyObject)]
#[expect(dead_code)]
enum ManyTypes {
    Int(i32),
    Bytes(Vec<u8>),
    String(String),
}

fn enum_from_pyobject(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let any = PyString::new(py, "hello world").into_any();

        b.iter(|| black_box(&any).extract::<ManyTypes>().unwrap());
    })
}

fn list_via_cast(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let any = PyList::empty(py).into_any();

        b.iter(|| black_box(&any).cast::<PyList>().unwrap());
    })
}

fn list_via_extract(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let any = PyList::empty(py).into_any();

        b.iter(|| black_box(&any).extract::<Bound<'_, PyList>>().unwrap());
    })
}

fn not_a_list_via_cast(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let any = PyString::new(py, "foobar").into_any();

        b.iter(|| black_box(&any).cast::<PyList>().unwrap_err());
    })
}

fn not_a_list_via_extract(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let any = PyString::new(py, "foobar").into_any();

        b.iter(|| black_box(&any).extract::<Bound<'_, PyList>>().unwrap_err());
    })
}

#[derive(FromPyObject)]
enum ListOrNotList<'a> {
    List(Bound<'a, PyList>),
    NotList(Bound<'a, PyAny>),
}

fn not_a_list_via_extract_enum(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let any = PyString::new(py, "foobar").into_any();

        b.iter(|| match black_box(&any).extract::<ListOrNotList<'_>>() {
            Ok(ListOrNotList::List(_list)) => panic!(),
            Ok(ListOrNotList::NotList(any)) => any,
            Err(_) => panic!(),
        });
    })
}

fn bench_vec_from_py_bytes(b: &mut Bencher<'_>, data: &[u8]) {
    Python::attach(|py| {
        let any = PyBytes::new(py, data).into_any();

        b.iter(|| black_box(&any).extract::<Vec<u8>>().unwrap());
    })
}

fn vec_bytes_from_py_bytes_small(b: &mut Bencher<'_>) {
    bench_vec_from_py_bytes(b, &[]);
}

fn vec_bytes_from_py_bytes_medium(b: &mut Bencher<'_>) {
    let data = (0..u8::MAX).collect::<Vec<u8>>();
    bench_vec_from_py_bytes(b, &data);
}

fn vec_bytes_from_py_bytes_large(b: &mut Bencher<'_>) {
    let data = vec![10u8; 100_000];
    bench_vec_from_py_bytes(b, &data);
}

fn bench_vec_from_py_bytearray(b: &mut Bencher<'_>, data: &[u8]) {
    Python::attach(|py| {
        let any = PyByteArray::new(py, data).into_any();

        b.iter(|| black_box(&any).extract::<Vec<u8>>().unwrap());
    })
}

fn vec_bytes_from_py_bytearray_small(b: &mut Bencher<'_>) {
    bench_vec_from_py_bytearray(b, &[]);
}

fn vec_bytes_from_py_bytearray_medium(b: &mut Bencher<'_>) {
    let data = (0..u8::MAX).collect::<Vec<u8>>();
    bench_vec_from_py_bytearray(b, &data);
}

fn vec_bytes_from_py_bytearray_large(b: &mut Bencher<'_>) {
    let data = vec![10u8; 100_000];
    bench_vec_from_py_bytearray(b, &data);
}

fn array_small_from_py_list(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let any = PyList::new(py, [1.0f64, 2.0, 3.0]).unwrap().into_any();

        b.iter(|| black_box(&any).extract::<[f64; 3]>().unwrap());
    })
}

fn array_large_from_py_list(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let any = PyList::new(py, 0..512i32).unwrap().into_any();

        b.iter(|| black_box(&any).extract::<[i64; 512]>().unwrap());
    })
}

fn array_large_from_py_tuple(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let any = PyTuple::new(py, 0..512i32).unwrap().into_any();

        b.iter(|| black_box(&any).extract::<[i64; 512]>().unwrap());
fn vec_ints_from_py_list_small(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let any = PyList::new(py, 0..3i32).unwrap().into_any();

        b.iter(|| black_box(&any).extract::<Vec<i64>>().unwrap());
    })
}

fn vec_ints_from_py_list(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let any = PyList::new(py, 0..1000i32).unwrap().into_any();

        b.iter(|| black_box(&any).extract::<Vec<i64>>().unwrap());
    })
}

fn vec_ints_from_py_tuple(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let any = PyTuple::new(py, 0..1000i32).unwrap().into_any();

        b.iter(|| black_box(&any).extract::<Vec<i64>>().unwrap());
    })
}

fn vec_strings_from_py_list(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let any = PyList::new(py, (0..1000).map(|i| format!("item-{i}")))
            .unwrap()
            .into_any();

        b.iter(|| black_box(&any).extract::<Vec<String>>().unwrap());
    })
}

// A sequence which is neither a list nor a tuple, so takes the generic path.
fn vec_ints_from_py_range(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let any = py.eval(c"range(1000)", None, None).unwrap();

        b.iter(|| black_box(&any).extract::<Vec<i64>>().unwrap());
    })
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("array_small_from_py_list", array_small_from_py_list);
    c.bench_function("array_large_from_py_list", array_large_from_py_list);
    c.bench_function("array_large_from_py_tuple", array_large_from_py_tuple);
    c.bench_function("vec_ints_from_py_list_small", vec_ints_from_py_list_small);
    c.bench_function("vec_ints_from_py_list", vec_ints_from_py_list);
    c.bench_function("vec_ints_from_py_tuple", vec_ints_from_py_tuple);
    c.bench_function("vec_strings_from_py_list", vec_strings_from_py_list);
    c.bench_function("vec_ints_from_py_range", vec_ints_from_py_range);

    c.bench_function("enum_from_pyobject", enum_from_pyobject);

    c.bench_function("list_via_cast", list_via_cast);

    c.bench_function("list_via_extract", list_via_extract);

    c.bench_function("not_a_list_via_cast", not_a_list_via_cast);
    c.bench_function("not_a_list_via_extract", not_a_list_via_extract);
    c.bench_function("not_a_list_via_extract_enum", not_a_list_via_extract_enum);

    c.bench_function(
        "vec_bytes_from_py_bytes_small",
        vec_bytes_from_py_bytes_small,
    );
    c.bench_function(
        "vec_bytes_from_py_bytes_medium",
        vec_bytes_from_py_bytes_medium,
    );
    c.bench_function(
        "vec_bytes_from_py_bytes_large",
        vec_bytes_from_py_bytes_large,
    );

    c.bench_function(
        "vec_bytes_from_py_bytearray_small",
        vec_bytes_from_py_bytearray_small,
    );
    c.bench_function(
        "vec_bytes_from_py_bytearray_medium",
        vec_bytes_from_py_bytearray_medium,
    );
    c.bench_function(
        "vec_bytes_from_py_bytearray_large",
        vec_bytes_from_py_bytearray_large,
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
