use crate::Bound;
use crate::platform::prelude::*;
use crate::types::PyString;
use crate::types::any::PyAnyMethods;
use crate::{
    FromPyObject, IntoPyObject, Py, PyAny, PyErr, PyErrArguments, PyResult, PyTypeInfo, Python,
    exceptions::PyTypeError, ffi,
};

#[cold]
pub fn failed_to_extract_enum(
    py: Python<'_>,
    type_name: &str,
    variant_names: &[&str],
    error_names: &[&str],
    errors: &[PyErr],
) -> PyErr {
    // TODO maybe use ExceptionGroup on Python 3.11+ ?
    let mut err_msg = format!(
        "failed to extract enum {} ('{}')",
        type_name,
        error_names.join(" | ")
    );
    for ((variant_name, error_name), error) in variant_names.iter().zip(error_names).zip(errors) {
        use core::fmt::Write;
        write!(
            &mut err_msg,
            "\n- variant {variant_name} ({error_name}): {error_msg}",
            variant_name = variant_name,
            error_name = error_name,
            error_msg = extract_traceback(py, error.clone_ref(py)),
        )
        .unwrap();
    }
    PyTypeError::new_err(err_msg)
}

/// Flattens a chain of errors into a single string.
fn extract_traceback(py: Python<'_>, mut error: PyErr) -> String {
    use core::fmt::Write;

    let mut error_msg = error.to_string();
    while let Some(cause) = error.cause(py) {
        write!(&mut error_msg, ", caused by {cause}").unwrap();
        error = cause
    }
    error_msg
}

/// Looks up `obj.<attr_name>` for a `#[pyo3(default)]` field.
///
/// Returns `None` if the lookup fails for any reason, so that the caller falls back to the
/// default value. A missing attribute does not create an `AttributeError` on Python 3.13+.
#[inline]
pub fn get_optional_attr<'py>(
    obj: &Bound<'py, PyAny>,
    attr_name: &Bound<'py, PyString>,
) -> Option<Bound<'py, PyAny>> {
    obj.getattr_opt(attr_name).ok().flatten()
}

/// Looks up `obj[key]` for a `#[pyo3(default)]` field.
///
/// Returns `None` if the lookup fails for any reason, so that the caller falls back to the
/// default value. A missing key does not create a `KeyError` for exact `dict`s on Python 3.13+.
#[inline]
pub fn get_optional_item<'py, K>(obj: &Bound<'py, PyAny>, key: K) -> Option<Bound<'py, PyAny>>
where
    K: IntoPyObject<'py>,
{
    #[cfg(Py_3_13)]
    {
        use crate::ffi_ptr_ext::FfiPtrExt;
        use crate::{BoundObject, IntoPyObjectExt, ffi};

        let py = obj.py();
        let key = key.into_pyobject_or_pyerr(py).ok()?;
        let mut result: *mut ffi::PyObject = core::ptr::null_mut();
        // SAFETY: `obj` and `key` are valid objects and we are attached to the interpreter;
        // `result` is a valid out-pointer.
        match unsafe { ffi::PyMapping_GetOptionalItem(obj.as_ptr(), key.as_ptr(), &mut result) } {
            // SAFETY: on success `result` is a new (non-null) strong reference
            1 => Some(unsafe { result.assume_owned_unchecked(py) }),
            // Item not found (KeyError suppressed)
            0 => None,
            // Some other error occurred; fetch it (which re-raises Rust panics) and discard it
            _ => {
                drop(PyErr::fetch(py));
                None
            }
        }
    }
    #[cfg(not(Py_3_13))]
    {
        obj.get_item(key).ok()
    }
}

pub fn extract_struct_field<'a, 'py, T>(
    obj: &'a Bound<'py, PyAny>,
    struct_name: &'static str,
    field_name: &'static str,
) -> PyResult<T>
where
    T: FromPyObject<'a, 'py>,
{
    match obj.extract() {
        Ok(value) => Ok(value),
        Err(err) => Err(failed_to_extract_struct_field(
            obj.py(),
            err.into(),
            struct_name,
            field_name,
        )),
    }
}

pub fn extract_struct_field_with<'a, 'py, T>(
    extractor: fn(&'a Bound<'py, PyAny>) -> PyResult<T>,
    obj: &'a Bound<'py, PyAny>,
    struct_name: &'static str,
    field_name: &'static str,
) -> PyResult<T> {
    match extractor(obj) {
        Ok(value) => Ok(value),
        Err(err) => Err(failed_to_extract_struct_field(
            obj.py(),
            err,
            struct_name,
            field_name,
        )),
    }
}

#[cold]
fn failed_to_extract_struct_field(
    _py: Python<'_>,
    inner_err: PyErr,
    struct_name: &'static str,
    field_name: &'static str,
) -> PyErr {
    PyTypeError::new_err(FailedToExtractField {
        struct_name,
        field: FieldName::Named(field_name),
        inner_err,
    })
}

pub fn extract_tuple_struct_field<'a, 'py, T>(
    obj: &'a Bound<'py, PyAny>,
    struct_name: &'static str,
    index: usize,
) -> PyResult<T>
where
    T: FromPyObject<'a, 'py>,
{
    match obj.extract() {
        Ok(value) => Ok(value),
        Err(err) => Err(failed_to_extract_tuple_struct_field(
            obj.py(),
            err.into(),
            struct_name,
            index,
        )),
    }
}

pub fn extract_tuple_struct_field_with<'a, 'py, T>(
    extractor: fn(&'a Bound<'py, PyAny>) -> PyResult<T>,
    obj: &'a Bound<'py, PyAny>,
    struct_name: &'static str,
    index: usize,
) -> PyResult<T> {
    match extractor(obj) {
        Ok(value) => Ok(value),
        Err(err) => Err(failed_to_extract_tuple_struct_field(
            obj.py(),
            err,
            struct_name,
            index,
        )),
    }
}

#[cold]
fn failed_to_extract_tuple_struct_field(
    _py: Python<'_>,
    inner_err: PyErr,
    struct_name: &'static str,
    index: usize,
) -> PyErr {
    PyTypeError::new_err(FailedToExtractField {
        struct_name,
        field: FieldName::Index(index),
        inner_err,
    })
}

enum FieldName {
    Named(&'static str),
    Index(usize),
}

/// Arguments for a lazily-constructed
/// `TypeError("failed to extract field {struct_name}.{field}")` whose `__cause__` is `inner_err`.
///
/// `derive(FromPyObject)` on enums tries each variant in turn and drops the errors of the
/// variants that failed once a later variant matches. Building the wrapping exception lazily
/// means those dropped errors never create (or normalize) any Python exception objects.
struct FailedToExtractField {
    struct_name: &'static str,
    field: FieldName,
    inner_err: PyErr,
}

impl PyErrArguments for FailedToExtractField {
    fn arguments(self, py: Python<'_>) -> Py<PyAny> {
        let msg = match self.field {
            FieldName::Named(field_name) => {
                format!(
                    "failed to extract field {}.{}",
                    self.struct_name, field_name
                )
            }
            FieldName::Index(index) => {
                format!("failed to extract field {}.{}", self.struct_name, index)
            }
        };
        // Return an already-constructed `TypeError` instance with its cause set: when the lazy
        // state is raised, `PyErr_SetObject(TypeError, value)` uses an instance of the exception
        // type as-is instead of calling the type with it as an argument.
        let value = match PyTypeError::type_object(py).call1((msg,)) {
            Ok(value) => value,
            // Creating a `TypeError` should not fail (barring e.g. `MemoryError`); if it does,
            // raise that failure instead.
            Err(err) => return err.into_value(py).into_any(),
        };
        let cause = self.inner_err.into_value(py);
        // Safety: `value` is an exception instance and `cause` is an owned exception instance;
        // `PyException_SetCause` steals the reference to `cause`.
        unsafe { ffi::PyException_SetCause(value.as_ptr(), cause.into_ptr()) };
        value.unbind()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IntoPyObject;
    use crate::exceptions::PyValueError;
    use core::sync::atomic::{AtomicUsize, Ordering};

    /// Lazy error arguments that count how many times they were materialized.
    struct CountingArgs(&'static AtomicUsize);

    impl PyErrArguments for CountingArgs {
        fn arguments(self, py: Python<'_>) -> Py<PyAny> {
            self.0.fetch_add(1, Ordering::SeqCst);
            "inner".into_pyobject(py).unwrap().into_any().unbind()
        }
    }

    fn check_wrapped(py: Python<'_>, err: PyErr, expected_msg: &str) {
        assert!(err.is_instance_of::<PyTypeError>(py));
        assert_eq!(err.value(py).to_string(), expected_msg);
        let cause = err.cause(py).expect("cause should be set");
        assert!(cause.is_instance_of::<PyValueError>(py));
        assert_eq!(cause.value(py).to_string(), "inner");
        assert!(
            err.value(py)
                .getattr("__suppress_context__")
                .unwrap()
                .is_truthy()
                .unwrap()
        );
    }

    #[test]
    fn struct_field_error_is_lazy() {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        Python::attach(|py| {
            let inner = PyValueError::new_err(CountingArgs(&COUNT));
            let err = failed_to_extract_struct_field(py, inner, "Foo", "bar");
            // building the wrapper must not materialize the inner error
            assert_eq!(COUNT.load(Ordering::SeqCst), 0);
            drop(err);
            assert_eq!(COUNT.load(Ordering::SeqCst), 0);

            let inner = PyValueError::new_err(CountingArgs(&COUNT));
            let err = failed_to_extract_struct_field(py, inner, "Foo", "bar");
            check_wrapped(py, err, "failed to extract field Foo.bar");
            assert_eq!(COUNT.load(Ordering::SeqCst), 1);
        })
    }

    #[test]
    fn tuple_struct_field_error_is_lazy() {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        Python::attach(|py| {
            let inner = PyValueError::new_err(CountingArgs(&COUNT));
            let err = failed_to_extract_tuple_struct_field(py, inner, "Foo::Bar", 3);
            assert_eq!(COUNT.load(Ordering::SeqCst), 0);
            check_wrapped(py, err, "failed to extract field Foo::Bar.3");
            assert_eq!(COUNT.load(Ordering::SeqCst), 1);
        })
    }

    #[test]
    fn field_error_cause_is_inner_error_instance() {
        Python::attach(|py| {
            // an already-normalized inner error keeps its identity as the cause
            let inner = PyValueError::new_err("inner");
            let inner_value = inner.value(py).clone();
            let err = failed_to_extract_struct_field(py, inner, "Foo", "bar");
            let cause = err.cause(py).unwrap();
            assert!(cause.value(py).is(&inner_value));
        })
    }

    #[test]
    fn field_error_restore_and_fetch() {
        Python::attach(|py| {
            // raising the still-lazy error into the interpreter produces the same exception
            let inner = PyValueError::new_err("inner");
            let err = failed_to_extract_tuple_struct_field(py, inner, "Foo", 0);
            err.restore(py);
            let err = PyErr::fetch(py);
            check_wrapped(py, err, "failed to extract field Foo.0");
        })
    }
}
