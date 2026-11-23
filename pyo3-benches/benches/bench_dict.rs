use std::collections::{BTreeMap, HashMap};
use std::cell::Cell;
use std::hint::black_box;

use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::types::IntoPyDict;
use pyo3::{prelude::*, types::PyMapping};

fn iter_dict(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        const LEN: usize = 100_000;
        let dict = (0..LEN as u64)
            .map(|i| (i, i * 2))
            .into_py_dict(py)
            .unwrap();
        let mut sum = 0;
        b.iter(|| {
            for (k, _v) in &dict {
                let i: u64 = k.extract().unwrap();
                sum += i;
            }
        });
    })
}

fn dict_locked_for_each(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        const LEN: usize = 100_000;
        let dict = (0..LEN as u64)
            .map(|i| (i, i * 2))
            .into_py_dict(py)
            .unwrap();
        let sum = Cell::new(0);
        b.iter(|| {
            dict.locked_for_each(|k, _v| {
                let i: u64 = k.extract()?;
                sum.set(sum.get() + i);
                Ok(())
            })
            .unwrap();
        });
fn dict_len(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let dict = (0..100u64).map(|i| (i, i * 2)).into_py_dict(py).unwrap();
        b.iter(|| black_box(&dict).len());
    })
}

fn dict_new(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        const LEN: usize = 50_000;
        b.iter_with_large_drop(|| {
            (0..LEN as u64)
                .map(|i| (i, i * 2))
                .into_py_dict(py)
                .unwrap()
        });
    });
}

fn dict_get_item(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        const LEN: usize = 50_000;
        let dict = (0..LEN as u64)
            .map(|i| (i, i * 2))
            .into_py_dict(py)
            .unwrap();
        let mut sum = 0;
        b.iter(|| {
            for i in 0..LEN {
                sum += dict
                    .get_item(i)
                    .unwrap()
                    .unwrap()
                    .extract::<usize>()
                    .unwrap();
            }
        });
    });
}

fn extract_hashmap(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        const LEN: usize = 100_000;
        let dict = (0..LEN as u64)
            .map(|i| (i, i * 2))
            .into_py_dict(py)
            .unwrap()
            .into_any();
        b.iter(|| HashMap::<u64, u64>::extract(dict.as_borrowed()));
    });
}

fn extract_btreemap(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        const LEN: usize = 100_000;
        let dict = (0..LEN as u64)
            .map(|i| (i, i * 2))
            .into_py_dict(py)
            .unwrap()
            .into_any();
        b.iter(|| BTreeMap::<u64, u64>::extract(dict.as_borrowed()));
    });
}

fn extract_btreemap_small(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let dict = (0..3u64)
            .map(|i| (i, i * 2))
            .into_py_dict(py)
            .unwrap()
            .into_any();
        b.iter(|| BTreeMap::<u64, u64>::extract(dict.as_borrowed()));
    });
}

fn extract_btreemap_shuffled(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        const LEN: u64 = 100_000;
        // deterministic pseudo-random permutation of 0..LEN (multiplier coprime to LEN)
        let dict = (0..LEN)
            .map(|i| ((i * 7919) % LEN, i))
            .into_py_dict(py)
            .unwrap()
            .into_any();
        b.iter(|| BTreeMap::<u64, u64>::extract(dict.as_borrowed()));
    });
}

fn extract_hashbrown_map(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        const LEN: usize = 100_000;
        let dict = (0..LEN as u64)
            .map(|i| (i, i * 2))
            .into_py_dict(py)
            .unwrap()
            .into_any();
        b.iter(|| hashbrown::HashMap::<u64, u64>::extract(dict.as_borrowed()));
    });
}

fn mapping_from_dict(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        const LEN: usize = 100_000;
        let dict = &(0..LEN as u64)
            .map(|i| (i, i * 2))
            .into_py_dict(py)
            .unwrap();
        b.iter(|| black_box(dict).cast::<PyMapping>().unwrap());
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("iter_dict", iter_dict);
    c.bench_function("dict_locked_for_each", dict_locked_for_each);
    c.bench_function("dict_len", dict_len);
    c.bench_function("dict_new", dict_new);
    c.bench_function("dict_get_item", dict_get_item);
    c.bench_function("extract_hashmap", extract_hashmap);
    c.bench_function("extract_btreemap", extract_btreemap);
    c.bench_function("extract_btreemap_small", extract_btreemap_small);
    c.bench_function("extract_btreemap_shuffled", extract_btreemap_shuffled);
    c.bench_function("mapping_from_dict", mapping_from_dict);
    c.bench_function("extract_hashbrown_map", extract_hashbrown_map);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
