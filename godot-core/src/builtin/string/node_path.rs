/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{
    fmt,
    ops::{
        Bound::{Excluded, Included, Unbounded},
        RangeBounds,
    },
};

use godot_ffi as sys;
use godot_ffi::{ffi_methods, GodotFfi};

use crate::builtin::inner;

use super::{GString, StringName};

/// A pre-parsed scene tree path.
///
/// # Null bytes
///
/// Note that Godot ignores any bytes after a null-byte. This means that for instance `"hello, world!"` and `"hello, world!\0 ignored by Godot"`
/// will be treated as the same string if converted to a `NodePath`.
///
/// # All string types
///
/// | Intended use case | String type                                |
/// |-------------------|--------------------------------------------|
/// | General purpose   | [`GString`][crate::builtin::GString]       |
/// | Interned names    | [`StringName`][crate::builtin::StringName] |
/// | Scene-node paths  | **`NodePath`**                             |
pub struct NodePath {
    opaque: sys::types::OpaqueNodePath,
}

impl NodePath {
    fn from_opaque(opaque: sys::types::OpaqueNodePath) -> Self {
        Self { opaque }
    }

    /// Returns the number of node names in the path. Property subnames are not included.
    pub fn get_name_count(&self) -> u32 {
        self.as_inner()
            .get_name_count()
            .try_into()
            .expect("Godot name counts are non negative int")
    }

    /// Returns the node name indicated by `idx`, starting from 0. If `idx` is out of bounds, [`None`] is returned.
    /// See also [`NodePath::get_subname_count`] and [`NodePath::get_name_count`].
    pub fn get_name(&self, idx: u32) -> Option<StringName> {
        let inner = self.as_inner();
        let idx = idx as i64;
        // This check checks both data being empty (get_name_count returns 0) and index out of bounds.
        if idx >= inner.get_name_count() {
            None
        } else {
            Some(inner.get_name(idx))
        }
    }

    /// Returns the number of property names ("subnames") in the path. Each subname in the node path is listed after a colon character (`:`).
    pub fn get_subname_count(&self) -> u32 {
        self.as_inner()
            .get_subname_count()
            .try_into()
            .expect("Godot subname counts are non negative int")
    }

    /// Returns a 32-bit integer hash value representing the string.
    pub fn hash(&self) -> u32 {
        self.as_inner()
            .hash()
            .try_into()
            .expect("Godot hashes are uint32_t")
    }

    /// Returns the property name indicated by `idx`, starting from 0. If `idx` is out of bounds, [`None`] is returned.
    /// See also [`NodePath::get_subname_count`].
    pub fn get_subname(&self, idx: u32) -> Option<StringName> {
        let inner = self.as_inner();
        let idx = idx as i64;
        // This check checks both data being empty (get_subname_count returns 0) and index out of bounds.
        if idx >= inner.get_subname_count() {
            None
        } else {
            Some(inner.get_subname(idx))
        }
    }

    /// Returns the slice of the [`NodePath`] as a new [`NodePath`]
    pub fn slice(&self, range: impl RangeBounds<i64>) -> NodePath {
        self.as_inner().slice(
            match range.start_bound() {
                Excluded(&start) => {
                    if start == -1 {
                        // Default end from godot, since the start is excluded.
                        i32::MAX as i64
                    } else {
                        start + 1
                    }
                }
                Included(&start) => start,
                Unbounded => 0,
            },
            match range.end_bound() {
                Excluded(&end) => end,
                Included(&end) => {
                    if end == -1 {
                        // Default end from godot, since the end is excluded.
                        i32::MAX as i64
                    } else {
                        end + 1
                    }
                }
                // Default end from godot.
                Unbounded => i32::MAX as i64,
            },
        )
    }

    crate::meta::declare_arg_method! {
        /// Use as argument for an [`impl AsArg<GString|StringName>`][crate::meta::AsArg] parameter.
        ///
        /// This is a convenient way to convert arguments of similar string types.
        ///
        /// # Example
        /// [`PackedStringArray`][crate::builtin::PackedStringArray] can insert elements using `AsArg<GString>`, so let's pass a `NodePath`:
        /// ```no_run
        /// # use godot::prelude::*;
        /// let node_path = NodePath::from("Node2D/Label");
        ///
        /// let mut array = PackedStringArray::new();
        /// array.push(node_path.arg());
        /// ```
    }

    #[doc(hidden)]
    pub fn as_inner(&self) -> inner::InnerNodePath {
        inner::InnerNodePath::from_outer(self)
    }
}

// SAFETY:
// - `move_return_ptr`
//   Nothing special needs to be done beyond a `std::mem::swap` when returning a NodePath.
//   So we can just use `ffi_methods`.
//
// - `from_arg_ptr`
//   NodePaths are properly initialized through a `from_sys` call, but the ref-count should be
//   incremented as that is the callee's responsibility. Which we do by calling
//   `std::mem::forget(node_path.clone())`.
unsafe impl GodotFfi for NodePath {
    fn variant_type() -> sys::VariantType {
        sys::VariantType::NODE_PATH
    }

    ffi_methods! { type sys::GDExtensionTypePtr = *mut Opaque; .. }
}

crate::meta::impl_godot_as_self!(NodePath);

impl_builtin_traits! {
    for NodePath {
        Default => node_path_construct_default;
        Clone => node_path_construct_copy;
        Drop => node_path_destroy;
        Eq => node_path_operator_equal;
        // NodePath provides no < operator.
        Hash;
    }
}

impl fmt::Display for NodePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = GString::from(self);
        <GString as fmt::Display>::fmt(&string, f)
    }
}

/// Uses literal syntax from GDScript: `^"node_path"`
impl fmt::Debug for NodePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = GString::from(self);
        write!(f, "^\"{string}\"")
    }
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Conversion from/into other string-types

impl_rust_string_conv!(NodePath);

impl From<&str> for NodePath {
    fn from(s: &str) -> Self {
        GString::from(s).into()
    }
}

impl From<String> for NodePath {
    fn from(s: String) -> Self {
        GString::from(s).into()
    }
}

impl From<&String> for NodePath {
    fn from(s: &String) -> Self {
        GString::from(s).into()
    }
}

impl From<&GString> for NodePath {
    fn from(string: &GString) -> Self {
        unsafe {
            Self::new_with_uninit(|self_ptr| {
                let ctor = sys::builtin_fn!(node_path_from_string);
                let args = [string.sys()];
                ctor(self_ptr, args.as_ptr());
            })
        }
    }
}

impl From<GString> for NodePath {
    /// Converts this `GString` to a `NodePath`.
    ///
    /// This is identical to `NodePath::from(&string)`, and as such there is no performance benefit.
    fn from(string: GString) -> Self {
        Self::from(&string)
    }
}

impl From<&StringName> for NodePath {
    fn from(string_name: &StringName) -> Self {
        Self::from(GString::from(string_name))
    }
}

impl From<StringName> for NodePath {
    /// Converts this `StringName` to a `NodePath`.
    ///
    /// This is identical to `NodePath::from(&string_name)`, and as such there is no performance benefit.
    fn from(string_name: StringName) -> Self {
        Self::from(GString::from(string_name))
    }
}

#[cfg(feature = "serde")]
mod serialize {
    use super::*;
    use serde::de::{Error, Visitor};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::fmt::Formatter;

    // For "Available on crate feature `serde`" in docs. Cannot be inherited from module. Also does not support #[derive] (e.g. in Vector2).
    #[cfg_attr(published_docs, doc(cfg(feature = "serde")))]
    impl Serialize for NodePath {
        #[inline]
        fn serialize<S>(
            &self,
            serializer: S,
        ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
        where
            S: Serializer,
        {
            serializer.serialize_newtype_struct("NodePath", &*self.to_string())
        }
    }

    #[cfg_attr(published_docs, doc(cfg(feature = "serde")))]
    impl<'de> Deserialize<'de> for NodePath {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct NodePathVisitor;

            impl<'de> Visitor<'de> for NodePathVisitor {
                type Value = NodePath;

                fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                    formatter.write_str("a NodePath")
                }

                fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
                where
                    E: Error,
                {
                    Ok(NodePath::from(s))
                }

                fn visit_newtype_struct<D>(
                    self,
                    deserializer: D,
                ) -> Result<Self::Value, <D as Deserializer<'de>>::Error>
                where
                    D: Deserializer<'de>,
                {
                    deserializer.deserialize_str(self)
                }
            }

            deserializer.deserialize_newtype_struct("NodePath", NodePathVisitor)
        }
    }
}
