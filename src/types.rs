use std::marker::PhantomData;
use std::fmt;

use gccjit_sys;

use context::Context;
use context;
use object;
use object::{Object, ToObject};

use gccjit_sys::gcc_jit_types::*;

/// A representation of a type, as it is known to the JIT compiler.
/// Types can be created through the Typeable trait or they can
/// be created dynamically by composing Field types.
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct Type<'ctx> {
    marker: PhantomData<&'ctx Context<'ctx>>,
    ptr: *mut gccjit_sys::gcc_jit_type
}

impl<'ctx> ToObject<'ctx> for Type<'ctx> {
    fn to_object(&self) -> Object<'ctx> {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_type_as_object(self.ptr);
            object::from_ptr(ptr)
        }
    }
}

impl<'ctx> fmt::Debug for Type<'ctx> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let obj = self.to_object();
        obj.fmt(fmt)
    }
}

impl<'ctx> Type<'ctx> {
    /// Given a type T, creates a type to *T, a pointer to T.
    pub fn make_pointer(self) -> Type<'ctx> {
        unsafe {
            from_ptr(gccjit_sys::gcc_jit_type_get_pointer(self.ptr))
        }
    }

    /// Given a type T, creates a type of const T.
    pub fn make_const(self) -> Type<'ctx> {
        unsafe {
            from_ptr(gccjit_sys::gcc_jit_type_get_const(self.ptr))
        }
    }

    /// Given a type T, creates a new type of volatile T, which
    /// has the semantics of C's volatile.
    pub fn make_volatile(self) -> Type<'ctx> {
        unsafe {
            from_ptr(gccjit_sys::gcc_jit_type_get_volatile(self.ptr))
        }
    }
}

/// Typeable is a trait for types that have a corresponding type within
/// gccjit. This library implements this type for a variety of primitive types,
/// but it's also possible to implement this trait for more complex types
/// that will use the API on Context to construct analagous struct/union types.
pub trait Typeable {
    fn get_type<'a, 'ctx>(&'a Context<'ctx>) -> Type<'a>;
}

macro_rules! typeable_def {
    ($ty:ty, $expr:expr) => {
        impl Typeable for $ty {
            fn get_type<'a, 'ctx>(ctx: &'a Context<'ctx>) -> Type<'a> {
                unsafe {
                    let ctx_ptr = context::get_ptr(ctx);
                    let ptr = gccjit_sys::gcc_jit_context_get_type(ctx_ptr, $expr);
                    from_ptr(ptr)
                }
            }
        }
    }
}

typeable_def!((), GCC_JIT_TYPE_VOID);
typeable_def!(bool, GCC_JIT_TYPE_BOOL);
typeable_def!(char, GCC_JIT_TYPE_CHAR);
typeable_def!(i8, GCC_JIT_TYPE_SIGNED_CHAR);
typeable_def!(u8, GCC_JIT_TYPE_UNSIGNED_CHAR);
typeable_def!(i16, GCC_JIT_TYPE_SHORT);
typeable_def!(u16, GCC_JIT_TYPE_UNSIGNED_SHORT);
typeable_def!(i32, GCC_JIT_TYPE_INT);
typeable_def!(u32, GCC_JIT_TYPE_UNSIGNED_INT);
typeable_def!(i64, GCC_JIT_TYPE_LONG);
typeable_def!(u64, GCC_JIT_TYPE_UNSIGNED_LONG);
typeable_def!(f32, GCC_JIT_TYPE_FLOAT);
typeable_def!(f64, GCC_JIT_TYPE_DOUBLE);
typeable_def!(usize, GCC_JIT_TYPE_SIZE_T);

/// Specific implementations of Typeable for *mut T and *const T that
/// represent void* and const void*, respectively. These impls should
/// only be used to expose opaque pointers to gccjit, not to create
/// pointers that are not opaque to gcc. For that, the make_pointer
/// function should be used.
impl<T> Typeable for *mut T {
    fn get_type<'a, 'ctx>(ctx: &'a Context<'ctx>) -> Type<'a> {
        unsafe {
            let ctx_ptr = context::get_ptr(ctx);
            let ptr = gccjit_sys::gcc_jit_context_get_type(ctx_ptr, GCC_JIT_TYPE_VOID_PTR);
            from_ptr(ptr)
        }
    }
}

impl<T> Typeable for *const T {
    fn get_type<'a, 'ctx>(ctx: &'a Context<'ctx>) -> Type<'a> {
        unsafe {
            let ctx_ptr = context::get_ptr(ctx);
            let ptr = gccjit_sys::gcc_jit_context_get_type(ctx_ptr, GCC_JIT_TYPE_VOID_PTR);
            from_ptr(ptr).make_const()
        }
    }
}

pub unsafe fn from_ptr<'ctx>(ptr: *mut gccjit_sys::gcc_jit_type) -> Type<'ctx> {
    Type {
        marker: PhantomData,
        ptr: ptr
    }
}

pub unsafe fn get_ptr<'ctx>(ty: &Type<'ctx>) -> *mut gccjit_sys::gcc_jit_type {
    ty.ptr
}
