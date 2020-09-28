pub mod hir;
mod macros;
pub mod mir;
pub mod op;
pub mod value;

use value::*;

// Special NoneType to be used in macros
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NoneTy {
    None,
}

types! {
    #[derive(Clone, Copy, PartialEq)]
    pub struct Value,

    #[derive(Clone, Copy, PartialEq, PartialOrd)]
    pub enum ValueWrapper,

    #[derive(Clone, Copy, PartialEq, PartialOrd)]
    pub enum ValueTy {
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
        NONE(NoneTy) = 12,
    }
}

ops! {
    pub struct Value, pub enum ValueWrapper {
        Add, add, +: [F64, F32, I8, I16, I32, I64, U8, U16, U32, U64],
        Sub, sub, -: [F64, F32, I8, I16, I32, I64, U8, U16, U32, U64],
        Mul, mul, *: [F64, F32, I8, I16, I32, I64, U8, U16, U32, U64],
        Div, div, /: [F64, F32, I8, I16, I32, I64, U8, U16, U32, U64],
        Rem, rem, %: [I8, I16, I32, I64, U8, U16, U32, U64],
    }
}

implicit_cast_rules! {
    pub struct Value, pub enum ValueWrapper, pub enum ValueTy {
        F64:  [F64(f64), F32(f32)],
        F32:  [F64(f64), F32(f32)],
        I8:   [I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64)],
        I16:  [I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64)],
        I32:  [I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64)],
        I64:  [I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64)],
        U8:   [I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64)],
        U16:  [I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64)],
        U32:  [I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64)],
        U64:  [I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64)],
        BOOL: [],
        STR:  [],
        NONE: [],
    }
}

explicit_cast_rules! {
    pub struct Value, pub enum ValueWrapper, pub enum ValueTy {
        F64:  [F64(f64), F32(f32), I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64)],
        F32:  [F64(f64), F32(f32), I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64)],
        I8:   [F64(f64), F32(f32), I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64)],
        I16:  [F64(f64), F32(f32), I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64)],
        I32:  [F64(f64), F32(f32), I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64)],
        I64:  [F64(f64), F32(f32), I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64)],
        U8:   [F64(f64), F32(f32), I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64)],
        U16:  [F64(f64), F32(f32), I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64)],
        U32:  [F64(f64), F32(f32), I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64)],
        U64:  [F64(f64), F32(f32), I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64)],
        BOOL: [],
        STR:  [],
        NONE: [],
    }
}
