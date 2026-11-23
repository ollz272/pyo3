use crate::conversion::IntoPyObject;
use crate::exceptions::PyValueError;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::{PyStaticExpr, type_hint_identifier, type_hint_union};
use crate::sync::PyOnceLock;
use crate::types::any::PyAnyMethods;
use crate::types::string::PyStringMethods;
use crate::types::{PyBytes, PyBytesMethods, PyString, PyType};
use crate::{Borrowed, Bound, FromPyObject, Py, PyAny, PyErr, Python, intern};
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};

impl FromPyObject<'_, '_> for IpAddr {
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr = type_hint_union!(
        type_hint_identifier!("ipaddress", "IPv4Address"),
        type_hint_identifier!("ipaddress", "IPv6Address")
    );

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> Result<Self, Self::Error> {
        // Fast path: an exact `str` has no `.packed` attribute, so skip the failing
        // `getattr` (which would build and discard an `AttributeError`) and parse directly.
        if let Ok(s) = obj.cast_exact::<PyString>() {
            return s.to_cow()?.parse().map_err(PyValueError::new_err);
        }

        match obj.getattr(intern!(obj.py(), "packed")) {
            Ok(packed) => {
                // Fast path: `bytes` can be dispatched on length directly, avoiding the
                // `[u8; 4]` extraction failing (and building a discarded `ValueError`) for IPv6.
                if let Ok(bytes) = packed.cast::<PyBytes>() {
                    let bytes = bytes.as_bytes();
                    return if let Ok(packed) = <[u8; 4]>::try_from(bytes) {
                        Ok(IpAddr::V4(Ipv4Addr::from(packed)))
                    } else if let Ok(packed) = <[u8; 16]>::try_from(bytes) {
                        Ok(IpAddr::V6(Ipv6Addr::from(packed)))
                    } else {
                        Err(PyValueError::new_err("invalid packed length"))
                    };
                }
                if let Ok(packed) = packed.extract::<[u8; 4]>() {
                    Ok(IpAddr::V4(Ipv4Addr::from(packed)))
                } else if let Ok(packed) = packed.extract::<[u8; 16]>() {
                    Ok(IpAddr::V6(Ipv6Addr::from(packed)))
                } else {
                    Err(PyValueError::new_err("invalid packed length"))
                }
            }
            Err(_) => {
                // We don't have a .packed attribute, so we try to construct an IP from str().
                obj.str()?.to_cow()?.parse().map_err(PyValueError::new_err)
            }
        }
    }
}

impl<'py> IntoPyObject<'py> for Ipv4Addr {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = type_hint_identifier!("ipaddress", "IPv4Address");

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        static IPV4_ADDRESS: PyOnceLock<Py<PyType>> = PyOnceLock::new();
        IPV4_ADDRESS
            .import(py, "ipaddress", "IPv4Address")?
            .call1((u32::from_be_bytes(self.octets()),))
    }
}

impl<'py> IntoPyObject<'py> for &Ipv4Addr {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = Ipv4Addr::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for Ipv6Addr {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = type_hint_identifier!("ipaddress", "IPv6Address");

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        static IPV6_ADDRESS: PyOnceLock<Py<PyType>> = PyOnceLock::new();
        IPV6_ADDRESS
            .import(py, "ipaddress", "IPv6Address")?
            .call1((u128::from_be_bytes(self.octets()),))
    }
}

impl<'py> IntoPyObject<'py> for &Ipv6Addr {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = Ipv6Addr::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for IpAddr {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr =
        type_hint_union!(&Ipv4Addr::OUTPUT_TYPE, &Ipv6Addr::OUTPUT_TYPE);

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            IpAddr::V4(ip) => ip.into_pyobject(py),
            IpAddr::V6(ip) => ip.into_pyobject(py),
        }
    }
}

impl<'py> IntoPyObject<'py> for &IpAddr {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = IpAddr::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

#[cfg(test)]
mod test_ipaddr {
    use core::str::FromStr;

    use super::*;

    #[test]
    fn test_roundtrip() {
        Python::attach(|py| {
            fn roundtrip(py: Python<'_>, ip: &str) {
                let ip = IpAddr::from_str(ip).unwrap();
                let py_cls = if ip.is_ipv4() {
                    "IPv4Address"
                } else {
                    "IPv6Address"
                };

                let pyobj = ip.into_pyobject(py).unwrap();
                let repr = pyobj.repr().unwrap();
                let repr = repr.to_string_lossy();
                assert_eq!(repr, format!("{py_cls}('{ip}')"));

                let ip2: IpAddr = pyobj.extract().unwrap();
                assert_eq!(ip, ip2);
            }
            roundtrip(py, "127.0.0.1");
            roundtrip(py, "::1");
            roundtrip(py, "0.0.0.0");
        });
    }

    #[test]
    fn test_from_pystring() {
        Python::attach(|py| {
            let py_str = PyString::new(py, "0:0:0:0:0:0:0:1");
            let ip: IpAddr = py_str.extract().unwrap();
            assert_eq!(ip, IpAddr::from_str("::1").unwrap());

            let py_str = PyString::new(py, "invalid");
            assert!(py_str.extract::<IpAddr>().is_err());

            let py_str = PyString::new(py, "127.0.0.1");
            let ip: IpAddr = py_str.extract().unwrap();
            assert_eq!(ip, IpAddr::from_str("127.0.0.1").unwrap());
        });
    }

    #[test]
    fn test_packed_bytes_lengths() {
        use crate::ffi::c_str;
        use crate::types::PyAnyMethods;

        Python::attach(|py| {
            let eval = |code: &core::ffi::CStr| py.eval(code, None, None).unwrap();

            // ipaddress objects (bytes `packed`, lengths 4 and 16)
            let ip: IpAddr = eval(c_str!("__import__('ipaddress').IPv4Address('10.1.2.3')"))
                .extract()
                .unwrap();
            assert_eq!(ip, IpAddr::from_str("10.1.2.3").unwrap());
            let ip: IpAddr = eval(c_str!("__import__('ipaddress').IPv6Address('2001:db8::1')"))
                .extract()
                .unwrap();
            assert_eq!(ip, IpAddr::from_str("2001:db8::1").unwrap());

            // duck-typed `packed` of bytes with an invalid length
            let bad = eval(c_str!("type('X', (), {'packed': b'\\x01\\x02\\x03'})()"));
            let err = bad.extract::<IpAddr>().unwrap_err();
            assert!(err.is_instance_of::<PyValueError>(py));
            assert_eq!(
                err.value(py).str().unwrap().to_cow().unwrap(),
                "invalid packed length"
            );

            // non-bytes `packed` keeps using the generic extraction path
            let ba = eval(c_str!(
                "type('X', (), {'packed': bytearray(b'\\x7f\\x00\\x00\\x01')})()"
            ));
            assert_eq!(
                ba.extract::<IpAddr>().unwrap(),
                IpAddr::from_str("127.0.0.1").unwrap()
            );
            let lst = eval(c_str!("type('X', (), {'packed': [127, 0, 0, 1]})()"));
            assert_eq!(
                lst.extract::<IpAddr>().unwrap(),
                IpAddr::from_str("127.0.0.1").unwrap()
            );
            let bad_lst = eval(c_str!("type('X', (), {'packed': [1, 2, 3]})()"));
            assert!(bad_lst.extract::<IpAddr>().is_err());
        });
    }

    #[test]
    fn test_str_subclass_uses_packed() {
        use crate::ffi::c_str;
        use crate::types::PyAnyMethods;

        // A `str` subclass is not an exact `str`, so its `packed` attribute is still honoured.
        Python::attach(|py| {
            let obj = py
                .eval(
                    c_str!("type('S', (str,), {'packed': b'\\x01\\x02\\x03\\x04'})('ignored')"),
                    None,
                    None,
                )
                .unwrap();
            assert_eq!(
                obj.extract::<IpAddr>().unwrap(),
                IpAddr::from_str("1.2.3.4").unwrap()
            );
        });
    }
}
