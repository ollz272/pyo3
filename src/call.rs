//! Defines how Python calls are dispatched, see [`PyCallArgs`].for more information.

use crate::ffi_ptr_ext::FfiPtrExt as _;
use crate::types::{PyAnyMethods as _, PyDict, PyString, PyTuple};
use crate::{Borrowed, Bound, IntoPyObjectExt as _, Py, PyAny, PyResult, ffi};

pub(crate) mod private {
    use super::*;

    pub trait Sealed {}

    impl Sealed for () {}
    impl Sealed for Bound<'_, PyTuple> {}
    impl Sealed for &'_ Bound<'_, PyTuple> {}
    impl Sealed for Py<PyTuple> {}
    impl Sealed for &'_ Py<PyTuple> {}
    impl Sealed for Borrowed<'_, '_, PyTuple> {}
    pub struct Token;
}

/// This trait marks types that can be used as arguments to Python function
/// calls.
///
/// This trait is currently implemented for Rust tuple (up to a size of 12),
/// [`Bound<'py, PyTuple>`] and [`Py<PyTuple>`]. Custom types that are
/// convertible to `PyTuple` via `IntoPyObject` need to do so before passing it
/// to `call`.
///
/// This trait is not intended to used by downstream crates directly. As such it
/// has no publicly available methods and cannot be implemented outside of
/// `pyo3`. The corresponding public API is available through [`call`]
/// ([`call0`], [`call1`] and friends) on [`PyAnyMethods`].
///
/// # What is `PyCallArgs` used for?
/// `PyCallArgs` is used internally in `pyo3` to dispatch the Python calls in
/// the most optimal way for the current build configuration. Certain types,
/// such as Rust tuples, do allow the usage of a faster calling convention of
/// the Python interpreter (if available). More types that may take advantage
/// from this may be added in the future.
///
/// [`call0`]: crate::types::PyAnyMethods::call0
/// [`call1`]: crate::types::PyAnyMethods::call1
/// [`call`]: crate::types::PyAnyMethods::call
/// [`PyAnyMethods`]: crate::types::PyAnyMethods
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot used as a Python `call` argument",
    note = "`PyCallArgs` is implemented for Rust tuples, `Bound<'py, PyTuple>` and `Py<PyTuple>`",
    note = "if your type is convertible to `PyTuple` via `IntoPyObject`, call `<arg>.into_pyobject(py)` manually",
    note = "if you meant to pass the type as a single argument, wrap it in a 1-tuple, `(<arg>,)`"
)]
pub trait PyCallArgs<'py>: Sized + private::Sealed {
    #[doc(hidden)]
    fn call(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        kwargs: Borrowed<'_, 'py, PyDict>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>>;

    #[doc(hidden)]
    fn call_positional(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>>;

    #[doc(hidden)]
    fn call_method_positional(
        self,
        object: Borrowed<'_, 'py, PyAny>,
        method_name: Borrowed<'_, 'py, PyString>,
        _: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        object
            .getattr(method_name)
            .and_then(|method| method.call1(self))
    }
}

impl<'py> PyCallArgs<'py> for () {
    fn call(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        kwargs: Borrowed<'_, 'py, PyDict>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        let args = self.into_pyobject_or_pyerr(function.py())?;
        args.call(function, kwargs, token)
    }

    fn call_positional(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        _: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        // SAFETY: `function` is a valid object pointer and the GIL/attached state is held
        unsafe {
            ffi::compat::PyObject_CallNoArgs(function.as_ptr()).assume_owned_or_err(function.py())
        }
    }

    fn call_method_positional(
        self,
        object: Borrowed<'_, 'py, PyAny>,
        method_name: Borrowed<'_, 'py, PyString>,
        _: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        // SAFETY: `object` and `method_name` are valid object pointers and the
        // GIL/attached state is held
        unsafe {
            ffi::compat::PyObject_CallMethodNoArgs(object.as_ptr(), method_name.as_ptr())
                .assume_owned_or_err(object.py())
        }
    }
}

impl<'py> PyCallArgs<'py> for Bound<'py, PyTuple> {
    #[inline]
    fn call(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        kwargs: Borrowed<'_, 'py, PyDict>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.as_borrowed().call(function, kwargs, token)
    }

    #[inline]
    fn call_positional(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.as_borrowed().call_positional(function, token)
    }
}

impl<'py> PyCallArgs<'py> for &'_ Bound<'py, PyTuple> {
    #[inline]
    fn call(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        kwargs: Borrowed<'_, 'py, PyDict>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.as_borrowed().call(function, kwargs, token)
    }

    #[inline]
    fn call_positional(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.as_borrowed().call_positional(function, token)
    }
}

impl<'py> PyCallArgs<'py> for Py<PyTuple> {
    #[inline]
    fn call(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        kwargs: Borrowed<'_, 'py, PyDict>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.bind_borrowed(function.py())
            .call(function, kwargs, token)
    }

    #[inline]
    fn call_positional(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.bind_borrowed(function.py())
            .call_positional(function, token)
    }
}

impl<'py> PyCallArgs<'py> for &'_ Py<PyTuple> {
    #[inline]
    fn call(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        kwargs: Borrowed<'_, 'py, PyDict>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.bind_borrowed(function.py())
            .call(function, kwargs, token)
    }

    #[inline]
    fn call_positional(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        token: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.bind_borrowed(function.py())
            .call_positional(function, token)
    }
}

impl<'py> PyCallArgs<'py> for Borrowed<'_, 'py, PyTuple> {
    #[inline]
    fn call(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        kwargs: Borrowed<'_, 'py, PyDict>,
        _: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        // SAFETY: `function`, `self` and `kwargs` are valid pointers to a
        // callable, a tuple and a dict, and `PyObject_Call` returns a new
        // owned reference on success, or NULL with an exception set on
        // failure.
        unsafe {
            ffi::PyObject_Call(function.as_ptr(), self.as_ptr(), kwargs.as_ptr())
                .assume_owned_or_err(function.py())
        }
    }

    #[inline]
    fn call_positional(
        self,
        function: Borrowed<'_, 'py, PyAny>,
        _: private::Token,
    ) -> PyResult<Bound<'py, PyAny>> {
        // SAFETY: `function` and `self` are valid pointers to a callable and
        // a tuple, NULL is documented as a valid value for kwargs, and
        // `PyObject_Call` returns a new owned reference on success, or NULL
        // with an exception set on failure.
        unsafe {
            ffi::PyObject_Call(function.as_ptr(), self.as_ptr(), core::ptr::null_mut())
                .assume_owned_or_err(function.py())
        }
    }
}

#[cfg(test)]
#[cfg(feature = "macros")]
mod tests {
    use crate::{
        Py, pyfunction,
        types::{PyDict, PyTuple},
    };

    #[pyfunction(signature = (*args, **kwargs), crate = "crate")]
    fn args_kwargs(
        args: Py<PyTuple>,
        kwargs: Option<Py<PyDict>>,
    ) -> (Py<PyTuple>, Option<Py<PyDict>>) {
        (args, kwargs)
    }

    #[test]
    fn test_call() {
        use crate::{
            Py, Python,
            types::{IntoPyDict, PyAnyMethods, PyDict, PyTuple},
            wrap_pyfunction,
        };

        Python::attach(|py| {
            let f = wrap_pyfunction!(args_kwargs, py).unwrap();

            let args = PyTuple::new(py, [1, 2, 3]).unwrap();
            let kwargs = &[("foo", 1), ("bar", 2)].into_py_dict(py).unwrap();

            macro_rules! check_call {
                ($args:expr, $kwargs:expr) => {
                    let (a, k): (Py<PyTuple>, Py<PyDict>) = f
                        .call(args.clone(), Some(kwargs))
                        .unwrap()
                        .extract()
                        .unwrap();
                    assert!(a.is(&args));
                    assert!(k.is(kwargs));
                };
            }

            // Bound<'py, PyTuple>
            check_call!(args.clone(), kwargs);

            // &Bound<'py, PyTuple>
            check_call!(&args, kwargs);

            // Py<PyTuple>
            check_call!(args.clone().unbind(), kwargs);

            // &Py<PyTuple>
            check_call!(&args.as_unbound(), kwargs);

            // Borrowed<'_, '_, PyTuple>
            check_call!(args.as_borrowed(), kwargs);
        })
    }

    #[test]
    fn test_call_positional() {
        use crate::{
            Py, Python,
            types::{PyAnyMethods, PyNone, PyTuple},
            wrap_pyfunction,
        };

        Python::attach(|py| {
            let f = wrap_pyfunction!(args_kwargs, py).unwrap();

            let args = PyTuple::new(py, [1, 2, 3]).unwrap();

            macro_rules! check_call {
                ($args:expr, $kwargs:expr) => {
                    let (a, k): (Py<PyTuple>, Py<PyNone>) =
                        f.call1(args.clone()).unwrap().extract().unwrap();
                    assert!(a.is(&args));
                    assert!(k.is_none(py));
                };
            }

            // Bound<'py, PyTuple>
            check_call!(args.clone(), kwargs);

            // &Bound<'py, PyTuple>
            check_call!(&args, kwargs);

            // Py<PyTuple>
            check_call!(args.clone().unbind(), kwargs);

            // &Py<PyTuple>
            check_call!(args.as_unbound(), kwargs);

            // Borrowed<'_, '_, PyTuple>
            check_call!(args.as_borrowed(), kwargs);
        })
    }

    #[test]
    fn test_call_unit() {
        use alloc::string::String;

        use crate::{
            Python,
            exceptions::PyAttributeError,
            ffi::c_str,
            types::{PyAnyMethods, PyModule},
        };

        Python::attach(|py| {
            let module = PyModule::from_code(
                py,
                c_str!(
                    "
class Foo:
    def method(self):
        return ('method', self)

    def __getattr__(self, name):
        if name == 'dynamic':
            return lambda: 'dynamic'
        raise AttributeError(name)

def func():
    return 'func'

def needs_arg(a):
    pass
"
                ),
                c_str!("test_call_unit.py"),
                c_str!("test_call_unit"),
            )
            .unwrap();

            // free function
            let func = module.getattr("func").unwrap();
            assert_eq!(func.call1(()).unwrap().extract::<String>().unwrap(), "func");
            assert_eq!(
                func.call((), None).unwrap().extract::<String>().unwrap(),
                "func"
            );

            // errors from the callee still propagate
            let needs_arg = module.getattr("needs_arg").unwrap();
            assert!(needs_arg.call1(()).is_err());

            // methods
            let foo = module.getattr("Foo").unwrap().call0().unwrap();
            let (name, this): (String, crate::Bound<'_, crate::PyAny>) =
                foo.call_method1("method", ()).unwrap().extract().unwrap();
            assert_eq!(name, "method");
            assert!(this.is(&foo));
            assert!(
                foo.call_method("method", (), None)
                    .unwrap()
                    .extract::<(String, crate::Bound<'_, crate::PyAny>)>()
                    .unwrap()
                    .1
                    .is(&foo)
            );

            // attribute lookup semantics (`__getattr__`) are preserved
            assert_eq!(
                foo.call_method1("dynamic", ())
                    .unwrap()
                    .extract::<String>()
                    .unwrap(),
                "dynamic"
            );

            // missing method raises AttributeError
            let err = foo.call_method1("missing", ()).unwrap_err();
            assert!(err.is_instance_of::<PyAttributeError>(py));

            // non-callable attribute raises TypeError
            let err = module.call_method1("__name__", ()).unwrap_err();
            assert!(err.is_instance_of::<crate::exceptions::PyTypeError>(py));
        })
    }
}
