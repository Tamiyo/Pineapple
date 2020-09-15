// We want a u64 object that can store a raw ptr w/ a tag
// We want a 48bit ptr to raw memory of some arbitrary type T
// We want the ability to store a tag inside the payload
// We want the ability to cast T to ANY other type U

use core::marker::PhantomData;
use core::mem;
use core::ptr;
use std::alloc;

const TY_MASK: u64 = u64::max_value() >> 16;

macro_rules! types {
    (
        $( #[$meta:meta] )*
        $struct_vis:vis struct $struct:ident, $enum_vis:vis enum $enum:ident {
            $( $variant:ident($ty:ty) = $tag:expr, )+
        }
    ) => {

        $( #[$meta] )*
        #[repr(transparent)]
        $struct_vis struct $struct {
            pub value: ValueBox<$enum>,
        }

        impl $struct {
            pub fn get_ty(&self) -> ValueTy {
                ValueTy::from(self.value.fetch_ty())
            }
        }

        $( #[$meta] )*
        $enum_vis enum $enum {
            $( $variant($ty) ),+
        }

        #[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
        #[repr(u16)]
        $enum_vis enum ValueTy {
            $($variant = $tag,) +
        }

        impl From<u16> for ValueTy {
            fn from(tag_u16: u16) -> Self {
                match tag_u16 {
                    $(
                        $tag => ValueTy::$variant,
                    )+
                    _ => panic!(format!("no type tag from '{:?}'", tag_u16))
                }
            }
        }

        trait ValueInner: Sized {
            fn into_tagged_box(self) -> ValueBox<Self>;
        }

        impl ValueInner for $enum {
            fn into_tagged_box(self) -> ValueBox<Self> {
                #[allow(non_camel_case_types)]
                enum __tagged_box_enum_counter {
                    $( $variant ),+
                }

                match self {
                    $(
                        Self::$variant(value) => ValueBox::new(value, __tagged_box_enum_counter::$variant as _),
                    )+
                }
            }
        }

        impl From<$enum> for $struct {
            #[inline]
            fn from(variant: $enum) -> Self {
                Self {
                    value: variant.into_tagged_box(),
                }
            }
        }

        $(
            impl From<$ty> for $struct {
                #[inline]
                fn from(value: $ty) -> Self {
                    Self {
                        value: $enum::$variant(value).into_tagged_box(),
                    }
                }
            }
        )+
    };
}

types! {
#[derive(Debug, Clone, PartialOrd, PartialEq)]
    pub struct Value, pub enum ValueTyTy {
        F64(f64) = 0,
        F32(f32) = 1,
        I8(i8) = 2,
        I16(i16) = 3,
        I32(i32) = 4,
        I64(i64) = 5,
        U8(u8) = 6,
        U16(u16) = 7,
        U32(u32) = 8,
        U64(u64) = 9,
        BOOL(bool) = 10,
        STR(usize) = 11,
    }
}
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ValueBox<T> {
    tagged_ptr: u64,
    _marker: PhantomData<T>,
}

impl<T> ValueBox<T> {
    #[inline]
    pub fn new<U>(val: U, ty: u16) -> Self {
        let ptr = if mem::size_of::<U>() == 0 {
            ptr::NonNull::dangling().as_ptr()
        } else {
            let layout = alloc::Layout::new::<U>();

            unsafe {
                let ptr = alloc::alloc(layout) as *mut U;
                assert!(ptr as usize != 0);
                ptr.write(val);

                ptr
            }
        };

        // This is possibly unsafe... looking into
        let tagged_ptr = ptr as u64 | ((ty as u64) << 48);
        Self {
            tagged_ptr,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn as_ptr(self) -> *const T {
        self.strip_ty() as *const T
    }

    #[inline]
    pub fn as_mut_ptr(self) -> *mut T {
        self.strip_ty() as *mut T
    }

    #[inline]
    pub fn fetch_ty(&self) -> u16 {
        (self.tagged_ptr >> 48) as u16
    }

    #[inline]
    pub fn strip_ty(&self) -> u64 {
        self.tagged_ptr & TY_MASK
    }

    #[inline]
    pub fn fetch_value(&self) -> &T {
        unsafe { &*(self.strip_ty() as *const T) }
    }
}
