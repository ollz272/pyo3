#[cfg(feature = "experimental-inspect")]
use crate::inspect::{PyStaticExpr, type_hint_subscript};
#[cfg(feature = "experimental-inspect")]
use crate::type_object::PyTypeInfo;
use crate::{
    Borrowed, FromPyObject, PyAny, PyErr, Python,
    conversion::{FromPyObjectOwned, IntoPyObject},
    instance::Bound,
    types::{PyDict, any::PyAnyMethods, dict::PyDictMethods},
};
use alloc::vec::Vec;

use core::cmp;
#[cfg(wip_feature_std)]
use core::hash;

#[cfg(wip_feature_std)]
impl<'py, K, V, H> IntoPyObject<'py> for std::collections::HashMap<K, V, H>
where
    K: IntoPyObject<'py> + cmp::Eq + hash::Hash,
    V: IntoPyObject<'py>,
    H: hash::BuildHasher,
{
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr =
        type_hint_subscript!(PyDict::TYPE_HINT, K::OUTPUT_TYPE, V::OUTPUT_TYPE);

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new(py);
        for (k, v) in self {
            dict.set_item(k, v)?;
        }
        Ok(dict)
    }
}

#[cfg(wip_feature_std)]
impl<'a, 'py, K, V, H> IntoPyObject<'py> for &'a std::collections::HashMap<K, V, H>
where
    &'a K: IntoPyObject<'py> + cmp::Eq + hash::Hash,
    &'a V: IntoPyObject<'py>,
    H: hash::BuildHasher,
{
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr =
        type_hint_subscript!(PyDict::TYPE_HINT, <&K>::OUTPUT_TYPE, <&V>::OUTPUT_TYPE);

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new(py);
        for (k, v) in self {
            dict.set_item(k, v)?;
        }
        Ok(dict)
    }
}

impl<'py, K, V> IntoPyObject<'py> for alloc::collections::BTreeMap<K, V>
where
    K: IntoPyObject<'py> + cmp::Eq,
    V: IntoPyObject<'py>,
{
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr =
        type_hint_subscript!(PyDict::TYPE_HINT, K::OUTPUT_TYPE, V::OUTPUT_TYPE);

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new(py);
        for (k, v) in self {
            dict.set_item(k, v)?;
        }
        Ok(dict)
    }
}

impl<'a, 'py, K, V> IntoPyObject<'py> for &'a alloc::collections::BTreeMap<K, V>
where
    &'a K: IntoPyObject<'py> + cmp::Eq,
    &'a V: IntoPyObject<'py>,
    K: 'a,
    V: 'a,
{
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr =
        type_hint_subscript!(PyDict::TYPE_HINT, <&K>::OUTPUT_TYPE, <&V>::OUTPUT_TYPE);

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new(py);
        for (k, v) in self {
            dict.set_item(k, v)?;
        }
        Ok(dict)
    }
}

#[cfg(wip_feature_std)]
impl<'py, K, V, S> FromPyObject<'_, 'py> for std::collections::HashMap<K, V, S>
where
    K: FromPyObjectOwned<'py> + cmp::Eq + hash::Hash,
    V: FromPyObjectOwned<'py>,
    S: hash::BuildHasher + Default,
{
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr =
        type_hint_subscript!(&PyDict::TYPE_HINT, K::INPUT_TYPE, V::INPUT_TYPE);

    fn extract(ob: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        let dict = ob.cast::<PyDict>()?;
        let mut ret = std::collections::HashMap::with_capacity_and_hasher(dict.len(), S::default());
        for (k, v) in dict.iter() {
            ret.insert(
                k.extract().map_err(Into::into)?,
                v.extract().map_err(Into::into)?,
            );
        }
        Ok(ret)
    }
}

impl<'py, K, V> FromPyObject<'_, 'py> for alloc::collections::BTreeMap<K, V>
where
    K: FromPyObjectOwned<'py> + cmp::Ord,
    V: FromPyObjectOwned<'py>,
{
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr =
        type_hint_subscript!(PyDict::TYPE_HINT, K::INPUT_TYPE, V::INPUT_TYPE);

    fn extract(ob: Borrowed<'_, 'py, PyAny>) -> Result<Self, PyErr> {
        let dict = ob.cast::<PyDict>()?;
        let mut pairs = Vec::with_capacity(dict.len());
        for (k, v) in dict.iter() {
            pairs.push((
                k.extract().map_err(Into::into)?,
                v.extract().map_err(Into::into)?,
            ));
        }
        Ok(pairs.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeMap;
    use alloc::string::String;
    #[cfg(wip_feature_std)]
    use std::collections::HashMap;

    #[test]
    #[cfg(wip_feature_std)]
    fn test_hashmap_to_python() {
        Python::attach(|py| {
            let mut map = HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = (&map).into_pyobject(py).unwrap();

            assert_eq!(py_map.len(), 1);
            assert!(
                py_map
                    .get_item(1)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
                    == 1
            );
            assert_eq!(map, py_map.extract().unwrap());
        });
    }

    #[test]
    fn test_btreemap_to_python() {
        Python::attach(|py| {
            let mut map = BTreeMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = (&map).into_pyobject(py).unwrap();

            assert_eq!(py_map.len(), 1);
            assert!(
                py_map
                    .get_item(1)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
                    == 1
            );
            assert_eq!(map, py_map.extract().unwrap());
        });
    }

    #[test]
    #[cfg(wip_feature_std)]
    fn test_hashmap_into_python() {
        Python::attach(|py| {
            let mut map = HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_pyobject(py).unwrap();

            assert_eq!(py_map.len(), 1);
            assert!(
                py_map
                    .get_item(1)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
                    == 1
            );
        });
    }

    #[test]
    fn test_btreemap_into_py() {
        Python::attach(|py| {
            let mut map = BTreeMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_pyobject(py).unwrap();

            assert_eq!(py_map.len(), 1);
            assert!(
                py_map
                    .get_item(1)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
                    == 1
            );
        });
    }

    #[test]
    fn test_btreemap_extract() {
        Python::attach(|py| {
            // empty
            let dict = PyDict::new(py);
            assert!(dict.extract::<BTreeMap<i32, i32>>().unwrap().is_empty());

            // unsorted insertion order produces a sorted map
            for k in [5, 1, 4, 2, 3] {
                dict.set_item(k, k * 10).unwrap();
            }
            let map = dict.extract::<BTreeMap<i32, i32>>().unwrap();
            assert_eq!(
                map.into_iter().collect::<Vec<_>>(),
                [(1, 10), (2, 20), (3, 30), (4, 40), (5, 50)]
            );

            // errors in a key or a value are propagated
            assert!(dict.extract::<BTreeMap<String, i32>>().is_err());
            assert!(dict.extract::<BTreeMap<i32, String>>().is_err());

            // not a dict
            assert!(py.None().bind(py).extract::<BTreeMap<i32, i32>>().is_err());
        });
    }
}
