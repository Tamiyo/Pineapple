// Borrowing a modified tagged implementation from https://crates.io/crates/tagged-box
use std::alloc::Layout;
use std::mem::ManuallyDrop;
use std::{marker::PhantomData, mem};

type TypeSize = u16;

const FREE_WIDTH: usize = 16;
const POINTER_WIDTH: usize = 48;

const MAX_TY_SIZE: u16 = u16::MAX;
const MAX_PTR_SIZE: usize = usize::max_value() >> FREE_WIDTH;

const TY_MASK: usize = usize::max_value() >> 16;

pub trait ValueInner: Sized {
    fn into_tagged_box(self) -> ValueBox<Self>;
    fn from_tagged_box(tagged: ValueBox<Self>) -> Self;
}

pub trait ValueContainer {
    type Inner;

    fn into_inner(self) -> Self::Inner;
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
#[repr(transparent)]
pub struct ValueBox<T> {
    pub boxed: ValuePointer,
    _type: PhantomData<T>,
}

impl<T> ValueBox<T> {
    pub fn new<U>(val: U, ty_size: TypeSize) -> Self {
        let ptr = if mem::size_of::<U>() == 0 {
            std::ptr::NonNull::dangling().as_ptr()
        } else {
            let layout = Layout::new::<U>();

            // Safety: The allocation should be properly handled by alloc + layout,
            // and writing should be properly aligned, as the pointer came from the
            // global allocator
            unsafe {
                let ptr = std::alloc::alloc(layout) as *mut U;
                assert!(ptr as usize != 0);
                ptr.write(val);

                ptr
            }
        };

        Self {
            boxed: ValuePointer::new(ptr as usize, ty_size),
            _type: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn as_ref<U>(&self) -> &U {
        self.boxed.as_ref()
    }

    #[inline]
    pub const fn as_ptr<U>(&self) -> *const U {
        self.boxed.as_ptr() as *const U
    }

    #[inline]
    pub fn as_mut_ptr<U>(&mut self) -> *mut U {
        self.boxed.as_mut_ptr() as *mut U
    }

    #[inline]
    pub const fn ty(&self) -> TypeSize {
        self.boxed.ty()
    }

    #[inline]
    #[must_use]
    pub unsafe fn into_inner<U>(tagged: Self) -> U {
        let mut tagged = ManuallyDrop::new(tagged);
        tagged.as_mut_ptr::<U>().read()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ValuePointer {
    tagged_ptr: usize,
}

impl ValuePointer {
    pub fn new(ptr: usize, ty_size: TypeSize) -> Self {
        assert!(ty_size <= MAX_TY_SIZE);
        assert!(ptr <= MAX_PTR_SIZE);

        let tagged_ptr = ptr | ((ty_size as usize) << POINTER_WIDTH);

        Self { tagged_ptr }
    }

    #[inline]
    pub unsafe fn as_ref<T>(&self) -> &T {
        &*(Self::strip_ty(self.tagged_ptr) as *const T)
    }

    #[inline]
    pub const fn as_ptr<T>(self) -> *const T {
        Self::strip_ty(self.tagged_ptr) as *const T
    }

    #[inline]
    pub fn as_mut_ptr<T>(self) -> *mut T {
        Self::strip_ty(self.tagged_ptr) as *mut T
    }

    #[inline]
    pub const fn ty(self) -> TypeSize {
        Self::fetch_ty(self.tagged_ptr)
    }

    #[inline]
    pub const fn fetch_ty(pointer: usize) -> TypeSize {
        (pointer >> POINTER_WIDTH) as TypeSize
    }

    #[inline]
    pub const fn strip_ty(pointer: usize) -> usize {
        pointer & TY_MASK
    }
}
