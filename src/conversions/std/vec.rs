#[cfg(feature = "experimental-inspect")]
use crate::inspect::{PyStaticExpr, type_hint_identifier, type_hint_subscript};
use crate::platform::prelude::*;
use crate::{
    Borrowed, CastError, PyResult, PyTypeInfo,
    conversion::{FromPyObject, FromPyObjectOwned, FromPyObjectSequence, IntoPyObject},
    exceptions::PyTypeError,
    ffi,
    types::{PyAnyMethods, PyList, PyListMethods, PySequence, PyString, PyTuple, PyTupleMethods},
};
use crate::{Bound, PyAny, PyErr, Python};

impl<'py, T> IntoPyObject<'py> for Vec<T>
where
    T: IntoPyObject<'py>,
{
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = T::SEQUENCE_OUTPUT_TYPE;

    /// Turns [`Vec<u8>`] into [`PyBytes`], all other `T`s will be turned into a [`PyList`]
    ///
    /// [`PyBytes`]: crate::types::PyBytes
    /// [`PyList`]: crate::types::PyList
    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        T::owned_sequence_into_pyobject(self, py, crate::conversion::private::Token)
    }
}

impl<'a, 'py, T> IntoPyObject<'py> for &'a Vec<T>
where
    &'a T: IntoPyObject<'py>,
{
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = <&[T]>::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        // NB: we could actually not cast to `PyAny`, which would be nice for
        // `&Vec<u8>`, but that'd be inconsistent with the `IntoPyObject` impl
        // above which always returns a `PyAny` for `Vec<T>`.
        self.as_slice().into_pyobject(py).map(Bound::into_any)
    }
}

impl<'py, T> FromPyObject<'_, 'py> for Vec<T>
where
    T: FromPyObjectOwned<'py>,
{
    type Error = PyErr;

    // `SupportsGetItem` is the closest static approximation of `PySequence_Check`, and unlike
    // `collections.abc.Sequence` it also accepts e.g. NumPy arrays. Type checkers therefore also
    // accept `dict` and `str`, which extraction rejects at runtime.
    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr = type_hint_subscript!(
        type_hint_identifier!("_typeshed", "SupportsGetItem"),
        type_hint_identifier!("builtins", "int"),
        T::INPUT_TYPE
    );

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        if let Some(extractor) = T::sequence_extractor(obj, crate::conversion::private::Token) {
            return Ok(extractor.to_vec());
        }

        if obj.is_instance_of::<PyString>() {
            return Err(PyTypeError::new_err("Can't extract `str` to `Vec`"));
        }

        extract_sequence(obj)
    }
}

fn extract_sequence<'py, T>(obj: Borrowed<'_, 'py, PyAny>) -> PyResult<Vec<T>>
where
    T: FromPyObjectOwned<'py>,
{
    // Fast paths for exact lists and tuples, which avoid the overhead of the iterator protocol.
    // Subclasses may override `__iter__`, so they take the generic path below.
    if let Ok(list) = obj.cast_exact::<PyList>() {
        let mut v = Vec::with_capacity(list.len());
        // `find_map` is specialised on free-threaded builds to hold the list's critical
        // section for the whole loop, rather than taking it once per element as `next` does.
        let err = list.iter().find_map(|item| match item.extract::<T>() {
            Ok(x) => {
                v.push(x);
                None
            }
            Err(e) => Some(e.into()),
        });
        return match err {
            Some(err) => Err(err),
            None => Ok(v),
        };
    }
    if let Ok(tuple) = obj.cast_exact::<PyTuple>() {
        let mut v = Vec::with_capacity(tuple.len());
        for item in tuple.iter_borrowed() {
            v.push(item.extract::<T>().map_err(Into::into)?);
        }
        return Ok(v);
    }

    // Types that pass `PySequence_Check` usually implement enough of the sequence protocol
    // to support this function and if not, we will only fail extraction safely.
    // SAFETY: passing valid pointer to python API
    if unsafe { ffi::PySequence_Check(obj.as_ptr()) } == 0 {
        return Err(CastError::new(obj, PySequence::type_object(obj.py()).into_any()).into());
    }

    let mut v = Vec::with_capacity(obj.len().unwrap_or(0));
    for item in obj.try_iter()? {
        v.push(item?.extract::<T>().map_err(Into::into)?);
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use crate::Python;
    use crate::conversion::IntoPyObject;
    use crate::platform::prelude::*;
    use crate::types::{PyAnyMethods, PyBytes, PyBytesMethods, PyList};

    #[test]
    fn test_vec_intopyobject_impl() {
        Python::attach(|py| {
            let bytes: Vec<u8> = b"foobar".to_vec();
            let obj = bytes.clone().into_pyobject(py).unwrap();
            assert!(obj.is_instance_of::<PyBytes>());
            let obj = obj.cast_into::<PyBytes>().unwrap();
            assert_eq!(obj.as_bytes(), &bytes);

            let nums: Vec<u16> = vec![0, 1, 2, 3];
            let obj = nums.into_pyobject(py).unwrap();
            assert!(obj.is_instance_of::<PyList>());
        });
    }

    #[test]
    fn test_vec_reference_intopyobject_impl() {
        Python::attach(|py| {
            let bytes: Vec<u8> = b"foobar".to_vec();
            let obj = (&bytes).into_pyobject(py).unwrap();
            assert!(obj.is_instance_of::<PyBytes>());
            let obj = obj.cast_into::<PyBytes>().unwrap();
            assert_eq!(obj.as_bytes(), &bytes);

            let nums: Vec<u16> = vec![0, 1, 2, 3];
            let obj = (&nums).into_pyobject(py).unwrap();
            assert!(obj.is_instance_of::<PyList>());
        });
    }

    #[test]
    fn test_strings_cannot_be_extracted_to_vec() {
        Python::attach(|py| {
            let v = "London Calling";
            let ob = v.into_pyobject(py).unwrap();

            assert!(ob.extract::<Vec<String>>().is_err());
            assert!(ob.extract::<Vec<char>>().is_err());
        });
    }

    #[test]
    fn test_extract_bytes_to_vec() {
        Python::attach(|py| {
            let v: Vec<u8> = PyBytes::new(py, b"abc").extract().unwrap();
            assert_eq!(v, b"abc");
        });
    }

    #[test]
    fn test_extract_tuple_to_vec() {
        Python::attach(|py| {
            let v: Vec<i32> = py.eval(c"(1, 2)", None, None).unwrap().extract().unwrap();
            assert_eq!(v, [1, 2]);
        });
    }

    #[test]
    fn test_extract_list_to_vec() {
        Python::attach(|py| {
            let v: Vec<i32> = py.eval(c"[1, 2]", None, None).unwrap().extract().unwrap();
            assert_eq!(v, [1, 2]);

            let v: Vec<i32> = py.eval(c"[]", None, None).unwrap().extract().unwrap();
            assert!(v.is_empty());

            // errors from extracting elements are propagated
            let obj = py.eval(c"[1, 'a']", None, None).unwrap();
            assert!(obj.extract::<Vec<i32>>().is_err());
            let obj = py.eval(c"(1, 'a')", None, None).unwrap();
            assert!(obj.extract::<Vec<i32>>().is_err());

            // the first failing element's error is returned
            let obj = py.eval(c"[1, 'a', None]", None, None).unwrap();
            let err = obj.extract::<Vec<i32>>().unwrap_err();
            assert!(err.to_string().contains("'str'"), "{err}");

            // nested lists (nested critical sections on free-threaded builds)
            let v: Vec<Vec<i32>> = py
                .eval(c"[[1, 2], [], [3]]", None, None)
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(v, [vec![1, 2], vec![], vec![3]]);
        });
    }

    #[test]
    fn test_extract_list_to_vec_mutated_during_extraction() {
        Python::attach(|py| {
            // element extraction runs Python code (`__index__`) which mutates the list being
            // extracted; this must neither deadlock nor read past the list's current length
            let globals = crate::types::PyDict::new(py);
            py.run(
                c"
class ClearsList:
    def __index__(self):
        l.clear()
        return 1

l = []
l.extend([ClearsList(), 2, 3])
",
                Some(&globals),
                None,
            )
            .unwrap();
            let l = globals.as_any().get_item("l").unwrap();
            let v: Vec<i32> = l.extract().unwrap();
            assert_eq!(v, [1]);
        });
    }

    #[test]
    fn test_extract_list_subclass_to_vec_uses_iter() {
        Python::attach(|py| {
            // subclasses which override `__iter__` must not take the exact-type fast path
            let v: Vec<i32> = py
                .eval(
                    c"type('L', (list,), {'__iter__': lambda self: iter([7, 8])})([1])",
                    None,
                    None,
                )
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(v, [7, 8]);

            let v: Vec<i32> = py
                .eval(
                    c"type('T', (tuple,), {'__iter__': lambda self: iter([7, 8])})((1,))",
                    None,
                    None,
                )
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(v, [7, 8]);
        });
    }

    #[test]
    fn test_extract_range_to_vec() {
        Python::attach(|py| {
            let v: Vec<i32> = py
                .eval(c"range(1, 5)", None, None)
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(v, [1, 2, 3, 4]);
        });
    }

    #[test]
    fn test_extract_bytearray_to_vec() {
        Python::attach(|py| {
            let v: Vec<u8> = py
                .eval(c"bytearray(b'abc')", None, None)
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(v, b"abc");
        });
    }
}
