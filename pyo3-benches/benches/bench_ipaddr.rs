use std::hint::black_box;
use std::net::IpAddr;

use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::prelude::*;
use pyo3::types::PyString;

fn ipaddr_extract_str_v4(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let s = PyString::new(py, "127.0.0.1");
        b.iter(|| black_box(&s).extract::<IpAddr>().unwrap());
    })
}

fn ipaddr_extract_str_v6(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let s = PyString::new(py, "2001:db8::1");
        b.iter(|| black_box(&s).extract::<IpAddr>().unwrap());
    })
}

fn ipaddr_extract_ipaddress_v4(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let ip = py
            .import("ipaddress")
            .unwrap()
            .getattr("IPv4Address")
            .unwrap()
            .call1(("127.0.0.1",))
            .unwrap();
        b.iter(|| black_box(&ip).extract::<IpAddr>().unwrap());
    })
}

fn ipaddr_extract_ipaddress_v6(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let ip = py
            .import("ipaddress")
            .unwrap()
            .getattr("IPv6Address")
            .unwrap()
            .call1(("2001:db8::1",))
            .unwrap();
        b.iter(|| black_box(&ip).extract::<IpAddr>().unwrap());
    })
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("ipaddr_extract_str_v4", ipaddr_extract_str_v4);
    c.bench_function("ipaddr_extract_str_v6", ipaddr_extract_str_v6);
    c.bench_function("ipaddr_extract_ipaddress_v4", ipaddr_extract_ipaddress_v4);
    c.bench_function("ipaddr_extract_ipaddress_v6", ipaddr_extract_ipaddress_v6);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
