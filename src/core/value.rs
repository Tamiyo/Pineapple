use super::relop::RelOp;
use crate::bytecode::string_intern::get_string;
use crate::bytecode::{distancef32::DistanceF32, distancef64::DistanceF64};
use crate::core::binop::BinOp;
use core::fmt;
use lazy_static::lazy_static;

type TupleSize = usize;
type Sym = usize;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Type {
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    Int,

    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    UInt,

    Float32,
    Float64,

    Bool,

    Char,

    String,

    None,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Primitive {
    // Integer Primitives
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Int(isize),
    Int128(i128),

    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    UInt(usize),
    UInt128(u128),

    // Floating PoInt Primitives
    Float32(DistanceF32),
    Float64(DistanceF64),

    // Boolean Primitive
    Bool(bool),

    // Character Primitive
    Char(char),

    String(usize),

    // None Primitive
    None,
}

impl Primitive {
    pub fn try_inference_to(&self, new_type: &Type) -> Result<Primitive, ()> {
        match (self, new_type) {
            (Primitive::Int8(a), Type::Int8) => Ok(Primitive::Int8(*a as i8)),
            (Primitive::Int8(a), Type::Int16) => Ok(Primitive::Int16(*a as i16)),
            (Primitive::Int8(a), Type::Int32) => Ok(Primitive::Int32(*a as i32)),
            (Primitive::Int8(a), Type::Int64) => Ok(Primitive::Int64(*a as i64)),
            (Primitive::Int8(a), Type::Int) => Ok(Primitive::Int(*a as isize)),
            (Primitive::Int8(a), Type::Int128) => Ok(Primitive::Int128(*a as i128)),

            (Primitive::Int16(a), Type::Int8) => Ok(Primitive::Int8(*a as i8)),
            (Primitive::Int16(a), Type::Int16) => Ok(Primitive::Int16(*a as i16)),
            (Primitive::Int16(a), Type::Int32) => Ok(Primitive::Int32(*a as i32)),
            (Primitive::Int16(a), Type::Int64) => Ok(Primitive::Int64(*a as i64)),
            (Primitive::Int16(a), Type::Int) => Ok(Primitive::Int(*a as isize)),
            (Primitive::Int16(a), Type::Int128) => Ok(Primitive::Int128(*a as i128)),

            (Primitive::Int32(a), Type::Int8) => Ok(Primitive::Int8(*a as i8)),
            (Primitive::Int32(a), Type::Int16) => Ok(Primitive::Int16(*a as i16)),
            (Primitive::Int32(a), Type::Int32) => Ok(Primitive::Int32(*a as i32)),
            (Primitive::Int32(a), Type::Int64) => Ok(Primitive::Int64(*a as i64)),
            (Primitive::Int32(a), Type::Int) => Ok(Primitive::Int(*a as isize)),
            (Primitive::Int32(a), Type::Int128) => Ok(Primitive::Int128(*a as i128)),

            (Primitive::Int64(a), Type::Int8) => Ok(Primitive::Int8(*a as i8)),
            (Primitive::Int64(a), Type::Int16) => Ok(Primitive::Int16(*a as i16)),
            (Primitive::Int64(a), Type::Int32) => Ok(Primitive::Int32(*a as i32)),
            (Primitive::Int64(a), Type::Int64) => Ok(Primitive::Int64(*a as i64)),
            (Primitive::Int64(a), Type::Int) => Ok(Primitive::Int(*a as isize)),
            (Primitive::Int64(a), Type::Int128) => Ok(Primitive::Int128(*a as i128)),

            (Primitive::Int(a), Type::Int8) => Ok(Primitive::Int8(*a as i8)),
            (Primitive::Int(a), Type::Int16) => Ok(Primitive::Int16(*a as i16)),
            (Primitive::Int(a), Type::Int32) => Ok(Primitive::Int32(*a as i32)),
            (Primitive::Int(a), Type::Int64) => Ok(Primitive::Int64(*a as i64)),
            (Primitive::Int(a), Type::Int) => Ok(Primitive::Int(*a as isize)),
            (Primitive::Int(a), Type::Int128) => Ok(Primitive::Int128(*a as i128)),

            (Primitive::Int128(a), Type::Int8) => Ok(Primitive::Int8(*a as i8)),
            (Primitive::Int128(a), Type::Int16) => Ok(Primitive::Int16(*a as i16)),
            (Primitive::Int128(a), Type::Int32) => Ok(Primitive::Int32(*a as i32)),
            (Primitive::Int128(a), Type::Int64) => Ok(Primitive::Int64(*a as i64)),
            (Primitive::Int128(a), Type::Int) => Ok(Primitive::Int(*a as isize)),
            (Primitive::Int128(a), Type::Int128) => Ok(Primitive::Int128(*a as i128)),

            (Primitive::UInt8(a), Type::UInt8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::UInt8(a), Type::UInt16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::UInt8(a), Type::UInt32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::UInt8(a), Type::UInt64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::UInt8(a), Type::UInt) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::UInt8(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),

            (Primitive::UInt16(a), Type::UInt8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::UInt16(a), Type::UInt16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::UInt16(a), Type::UInt32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::UInt16(a), Type::UInt64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::UInt16(a), Type::UInt) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::UInt16(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),

            (Primitive::UInt32(a), Type::UInt8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::UInt32(a), Type::UInt16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::UInt32(a), Type::UInt32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::UInt32(a), Type::UInt64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::UInt32(a), Type::UInt) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::UInt32(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),

            (Primitive::UInt64(a), Type::UInt8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::UInt64(a), Type::UInt16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::UInt64(a), Type::UInt32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::UInt64(a), Type::UInt64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::UInt64(a), Type::UInt) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::UInt64(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),

            (Primitive::UInt(a), Type::UInt8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::UInt(a), Type::UInt16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::UInt(a), Type::UInt32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::UInt(a), Type::UInt64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::UInt(a), Type::UInt) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::UInt(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),

            (Primitive::UInt128(a), Type::UInt8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::UInt128(a), Type::UInt16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::UInt128(a), Type::UInt32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::UInt128(a), Type::UInt64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::UInt128(a), Type::UInt) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::UInt128(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),

            (Primitive::Float64(a), Type::Float32) => {
                let af64: f64 = a.into();
                Ok(Primitive::Float32(DistanceF32::from(af64 as f32)))
            }
            (Primitive::Float64(a), Type::Float64) => Ok(Primitive::Float64(*a)),

            (Primitive::Float32(a), Type::Float32) => Ok(Primitive::Float32(*a)),
            (Primitive::Float32(a), Type::Float64) => {
                let af32: f32 = a.into();
                Ok(Primitive::Float64(DistanceF64::from(af32 as f64)))
            }

            _ => Err(()),
        }
    }

    pub fn try_cast_to(&self, new_type: &Type) -> Result<Primitive, ()> {
        match (self, new_type) {
            (Primitive::Int8(a), Type::Int8) => Ok(Primitive::Int8(*a as i8)),
            (Primitive::Int8(a), Type::Int16) => Ok(Primitive::Int16(*a as i16)),
            (Primitive::Int8(a), Type::Int32) => Ok(Primitive::Int32(*a as i32)),
            (Primitive::Int8(a), Type::Int64) => Ok(Primitive::Int64(*a as i64)),
            (Primitive::Int8(a), Type::Int) => Ok(Primitive::Int(*a as isize)),
            (Primitive::Int8(a), Type::Int128) => Ok(Primitive::Int128(*a as i128)),
            (Primitive::Int8(a), Type::UInt8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::Int8(a), Type::UInt16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::Int8(a), Type::UInt32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::Int8(a), Type::UInt64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::Int8(a), Type::UInt) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::Int8(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),
            (Primitive::Int8(a), Type::Float32) => {
                Ok(Primitive::Float32(DistanceF32::from(*a as f32)))
            }
            (Primitive::Int8(a), Type::Float64) => {
                Ok(Primitive::Float64(DistanceF64::from(*a as f64)))
            }

            (Primitive::Int16(a), Type::Int8) => Ok(Primitive::Int8(*a as i8)),
            (Primitive::Int16(a), Type::Int16) => Ok(Primitive::Int16(*a as i16)),
            (Primitive::Int16(a), Type::Int32) => Ok(Primitive::Int32(*a as i32)),
            (Primitive::Int16(a), Type::Int64) => Ok(Primitive::Int64(*a as i64)),
            (Primitive::Int16(a), Type::Int) => Ok(Primitive::Int(*a as isize)),
            (Primitive::Int16(a), Type::Int128) => Ok(Primitive::Int128(*a as i128)),
            (Primitive::Int16(a), Type::UInt8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::Int16(a), Type::UInt16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::Int16(a), Type::UInt32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::Int16(a), Type::UInt64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::Int16(a), Type::UInt) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::Int16(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),
            (Primitive::Int16(a), Type::Float32) => {
                Ok(Primitive::Float32(DistanceF32::from(*a as f32)))
            }
            (Primitive::Int16(a), Type::Float64) => {
                Ok(Primitive::Float64(DistanceF64::from(*a as f64)))
            }

            (Primitive::Int32(a), Type::Int8) => Ok(Primitive::Int8(*a as i8)),
            (Primitive::Int32(a), Type::Int16) => Ok(Primitive::Int16(*a as i16)),
            (Primitive::Int32(a), Type::Int32) => Ok(Primitive::Int32(*a as i32)),
            (Primitive::Int32(a), Type::Int64) => Ok(Primitive::Int64(*a as i64)),
            (Primitive::Int32(a), Type::Int) => Ok(Primitive::Int(*a as isize)),
            (Primitive::Int32(a), Type::Int128) => Ok(Primitive::Int128(*a as i128)),
            (Primitive::Int32(a), Type::UInt8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::Int32(a), Type::UInt16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::Int32(a), Type::UInt32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::Int32(a), Type::UInt64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::Int32(a), Type::UInt) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::Int32(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),
            (Primitive::Int32(a), Type::Float32) => {
                Ok(Primitive::Float32(DistanceF32::from(*a as f32)))
            }
            (Primitive::Int32(a), Type::Float64) => {
                Ok(Primitive::Float64(DistanceF64::from(*a as f64)))
            }

            (Primitive::Int64(a), Type::Int8) => Ok(Primitive::Int8(*a as i8)),
            (Primitive::Int64(a), Type::Int16) => Ok(Primitive::Int16(*a as i16)),
            (Primitive::Int64(a), Type::Int32) => Ok(Primitive::Int32(*a as i32)),
            (Primitive::Int64(a), Type::Int64) => Ok(Primitive::Int64(*a as i64)),
            (Primitive::Int64(a), Type::Int) => Ok(Primitive::Int(*a as isize)),
            (Primitive::Int64(a), Type::Int128) => Ok(Primitive::Int128(*a as i128)),
            (Primitive::Int64(a), Type::UInt8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::Int64(a), Type::UInt16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::Int64(a), Type::UInt32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::Int64(a), Type::UInt64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::Int64(a), Type::UInt) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::Int64(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),
            (Primitive::Int64(a), Type::Float32) => {
                Ok(Primitive::Float32(DistanceF32::from(*a as f32)))
            }
            (Primitive::Int64(a), Type::Float64) => {
                Ok(Primitive::Float64(DistanceF64::from(*a as f64)))
            }

            (Primitive::Int(a), Type::Int8) => Ok(Primitive::Int8(*a as i8)),
            (Primitive::Int(a), Type::Int16) => Ok(Primitive::Int16(*a as i16)),
            (Primitive::Int(a), Type::Int32) => Ok(Primitive::Int32(*a as i32)),
            (Primitive::Int(a), Type::Int64) => Ok(Primitive::Int64(*a as i64)),
            (Primitive::Int(a), Type::Int) => Ok(Primitive::Int(*a as isize)),
            (Primitive::Int(a), Type::Int128) => Ok(Primitive::Int128(*a as i128)),
            (Primitive::Int(a), Type::UInt8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::Int(a), Type::UInt16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::Int(a), Type::UInt32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::Int(a), Type::UInt64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::Int(a), Type::UInt) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::Int(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),
            (Primitive::Int(a), Type::Float32) => {
                Ok(Primitive::Float32(DistanceF32::from(*a as f32)))
            }
            (Primitive::Int(a), Type::Float64) => {
                Ok(Primitive::Float64(DistanceF64::from(*a as f64)))
            }

            (Primitive::Int128(a), Type::Int8) => Ok(Primitive::Int8(*a as i8)),
            (Primitive::Int128(a), Type::Int16) => Ok(Primitive::Int16(*a as i16)),
            (Primitive::Int128(a), Type::Int32) => Ok(Primitive::Int32(*a as i32)),
            (Primitive::Int128(a), Type::Int64) => Ok(Primitive::Int64(*a as i64)),
            (Primitive::Int128(a), Type::Int) => Ok(Primitive::Int(*a as isize)),
            (Primitive::Int128(a), Type::Int128) => Ok(Primitive::Int128(*a as i128)),
            (Primitive::Int128(a), Type::UInt8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::Int128(a), Type::UInt16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::Int128(a), Type::UInt32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::Int128(a), Type::UInt64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::Int128(a), Type::UInt) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::Int128(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),
            (Primitive::Int128(a), Type::Float32) => {
                Ok(Primitive::Float32(DistanceF32::from(*a as f32)))
            }
            (Primitive::Int128(a), Type::Float64) => {
                Ok(Primitive::Float64(DistanceF64::from(*a as f64)))
            }

            (Primitive::UInt8(a), Type::UInt8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::UInt8(a), Type::UInt16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::UInt8(a), Type::UInt32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::UInt8(a), Type::UInt64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::UInt8(a), Type::UInt) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::UInt8(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),
            (Primitive::UInt8(a), Type::Int8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::UInt8(a), Type::Int16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::UInt8(a), Type::Int32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::UInt8(a), Type::Int64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::UInt8(a), Type::Int) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::UInt8(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),
            (Primitive::UInt8(a), Type::Float32) => {
                Ok(Primitive::Float32(DistanceF32::from(*a as f32)))
            }
            (Primitive::UInt8(a), Type::Float64) => {
                Ok(Primitive::Float64(DistanceF64::from(*a as f64)))
            }

            (Primitive::UInt16(a), Type::UInt8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::UInt16(a), Type::UInt16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::UInt16(a), Type::UInt32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::UInt16(a), Type::UInt64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::UInt16(a), Type::UInt) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::UInt16(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),
            (Primitive::UInt16(a), Type::Int8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::UInt16(a), Type::Int16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::UInt16(a), Type::Int32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::UInt16(a), Type::Int64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::UInt16(a), Type::Int) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::UInt16(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),
            (Primitive::UInt16(a), Type::Float32) => {
                Ok(Primitive::Float32(DistanceF32::from(*a as f32)))
            }
            (Primitive::UInt16(a), Type::Float64) => {
                Ok(Primitive::Float64(DistanceF64::from(*a as f64)))
            }

            (Primitive::UInt32(a), Type::UInt8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::UInt32(a), Type::UInt16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::UInt32(a), Type::UInt32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::UInt32(a), Type::UInt64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::UInt32(a), Type::UInt) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::UInt32(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),
            (Primitive::UInt32(a), Type::Int8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::UInt32(a), Type::Int16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::UInt32(a), Type::Int32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::UInt32(a), Type::Int64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::UInt32(a), Type::Int) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::UInt32(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),
            (Primitive::UInt32(a), Type::Float32) => {
                Ok(Primitive::Float32(DistanceF32::from(*a as f32)))
            }
            (Primitive::UInt32(a), Type::Float64) => {
                Ok(Primitive::Float64(DistanceF64::from(*a as f64)))
            }

            (Primitive::UInt64(a), Type::UInt8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::UInt64(a), Type::UInt16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::UInt64(a), Type::UInt32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::UInt64(a), Type::UInt64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::UInt64(a), Type::UInt) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::UInt64(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),
            (Primitive::UInt64(a), Type::Int8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::UInt64(a), Type::Int16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::UInt64(a), Type::Int32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::UInt64(a), Type::Int64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::UInt64(a), Type::Int) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::UInt64(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),
            (Primitive::UInt64(a), Type::Float32) => {
                Ok(Primitive::Float32(DistanceF32::from(*a as f32)))
            }
            (Primitive::UInt64(a), Type::Float64) => {
                Ok(Primitive::Float64(DistanceF64::from(*a as f64)))
            }

            (Primitive::UInt(a), Type::UInt8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::UInt(a), Type::UInt16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::UInt(a), Type::UInt32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::UInt(a), Type::UInt64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::UInt(a), Type::UInt) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::UInt(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),
            (Primitive::UInt(a), Type::Int8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::UInt(a), Type::Int16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::UInt(a), Type::Int32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::UInt(a), Type::Int64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::UInt(a), Type::Int) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::UInt(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),
            (Primitive::UInt(a), Type::Float32) => {
                Ok(Primitive::Float32(DistanceF32::from(*a as f32)))
            }
            (Primitive::UInt(a), Type::Float64) => {
                Ok(Primitive::Float64(DistanceF64::from(*a as f64)))
            }

            (Primitive::UInt128(a), Type::UInt8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::UInt128(a), Type::UInt16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::UInt128(a), Type::UInt32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::UInt128(a), Type::UInt64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::UInt128(a), Type::UInt) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::UInt128(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),
            (Primitive::UInt128(a), Type::Int8) => Ok(Primitive::UInt8(*a as u8)),
            (Primitive::UInt128(a), Type::Int16) => Ok(Primitive::UInt16(*a as u16)),
            (Primitive::UInt128(a), Type::Int32) => Ok(Primitive::UInt32(*a as u32)),
            (Primitive::UInt128(a), Type::Int64) => Ok(Primitive::UInt64(*a as u64)),
            (Primitive::UInt128(a), Type::Int) => Ok(Primitive::UInt(*a as usize)),
            (Primitive::UInt128(a), Type::UInt128) => Ok(Primitive::UInt128(*a as u128)),
            (Primitive::UInt128(a), Type::Float32) => {
                Ok(Primitive::Float32(DistanceF32::from(*a as f32)))
            }
            (Primitive::UInt128(a), Type::Float64) => {
                Ok(Primitive::Float64(DistanceF64::from(*a as f64)))
            }

            // Float64
            (Primitive::Float64(a), Type::Float32) => {
                let af64: f64 = a.into();
                Ok(Primitive::Float32(DistanceF32::from(af64 as f32)))
            }
            (Primitive::Float64(a), Type::Float64) => Ok(Primitive::Float64(*a)),
            (Primitive::Float64(a), Type::Int8) => {
                let af64: f64 = a.into();
                Ok(Primitive::Int8(af64 as i8))
            }
            (Primitive::Float64(a), Type::Int16) => {
                let af64: f64 = a.into();
                Ok(Primitive::Int16(af64 as i16))
            }
            (Primitive::Float64(a), Type::Int32) => {
                let af64: f64 = a.into();
                Ok(Primitive::Int32(af64 as i32))
            }
            (Primitive::Float64(a), Type::Int64) => {
                let af64: f64 = a.into();
                Ok(Primitive::Int64(af64 as i64))
            }
            (Primitive::Float64(a), Type::Int) => {
                let af64: f64 = a.into();
                Ok(Primitive::Int(af64 as isize))
            }
            (Primitive::Float64(a), Type::Int128) => {
                let af64: f64 = a.into();
                Ok(Primitive::Int128(af64 as i128))
            }
            (Primitive::Float64(a), Type::UInt8) => {
                let af64: f64 = a.into();
                Ok(Primitive::UInt8(af64 as u8))
            }
            (Primitive::Float64(a), Type::UInt16) => {
                let af64: f64 = a.into();
                Ok(Primitive::UInt16(af64 as u16))
            }
            (Primitive::Float64(a), Type::UInt32) => {
                let af64: f64 = a.into();
                Ok(Primitive::UInt32(af64 as u32))
            }
            (Primitive::Float64(a), Type::UInt64) => {
                let af64: f64 = a.into();
                Ok(Primitive::UInt64(af64 as u64))
            }
            (Primitive::Float64(a), Type::UInt) => {
                let af64: f64 = a.into();
                Ok(Primitive::UInt(af64 as usize))
            }
            (Primitive::Float64(a), Type::UInt128) => {
                let af64: f64 = a.into();
                Ok(Primitive::UInt128(af64 as u128))
            }

            // Float32
            (Primitive::Float32(a), Type::Float32) => Ok(Primitive::Float32(*a)),
            (Primitive::Float32(a), Type::Float64) => {
                let af32: f32 = a.into();
                Ok(Primitive::Float64(DistanceF64::from(af32 as f64)))
            }
            (Primitive::Float32(a), Type::Int8) => {
                let af32: f32 = a.into();
                Ok(Primitive::Int8(af32 as i8))
            }
            (Primitive::Float32(a), Type::Int16) => {
                let af32: f32 = a.into();
                Ok(Primitive::Int16(af32 as i16))
            }
            (Primitive::Float32(a), Type::Int32) => {
                let af32: f32 = a.into();
                Ok(Primitive::Int32(af32 as i32))
            }
            (Primitive::Float32(a), Type::Int64) => {
                let af32: f32 = a.into();
                Ok(Primitive::Int64(af32 as i64))
            }
            (Primitive::Float32(a), Type::Int) => {
                let af32: f32 = a.into();
                Ok(Primitive::Int(af32 as isize))
            }
            (Primitive::Float32(a), Type::Int128) => {
                let af32: f32 = a.into();
                Ok(Primitive::Int128(af32 as i128))
            }
            (Primitive::Float32(a), Type::UInt8) => {
                let af32: f32 = a.into();
                Ok(Primitive::UInt8(af32 as u8))
            }
            (Primitive::Float32(a), Type::UInt16) => {
                let af32: f32 = a.into();
                Ok(Primitive::UInt16(af32 as u16))
            }
            (Primitive::Float32(a), Type::UInt32) => {
                let af32: f32 = a.into();
                Ok(Primitive::UInt32(af32 as u32))
            }
            (Primitive::Float32(a), Type::UInt64) => {
                let af32: f32 = a.into();
                Ok(Primitive::UInt64(af32 as u64))
            }
            (Primitive::Float32(a), Type::UInt) => {
                let af32: f32 = a.into();
                Ok(Primitive::UInt(af32 as usize))
            }
            (Primitive::Float32(a), Type::UInt128) => {
                let af32: f32 = a.into();
                Ok(Primitive::UInt128(af32 as u128))
            }

            _ => Err(()),
        }
    }

    pub fn can_inference_to(&self, new_type: &Type) -> bool {
        match (self, new_type) {
            (Primitive::Int8(a), Type::Int8) => true,
            (Primitive::Int8(a), Type::Int16) => true,
            (Primitive::Int8(a), Type::Int32) => true,
            (Primitive::Int8(a), Type::Int64) => true,
            (Primitive::Int8(a), Type::Int) => true,
            (Primitive::Int8(a), Type::Int128) => true,

            (Primitive::Int16(a), Type::Int8) => true,
            (Primitive::Int16(a), Type::Int16) => true,
            (Primitive::Int16(a), Type::Int32) => true,
            (Primitive::Int16(a), Type::Int64) => true,
            (Primitive::Int16(a), Type::Int) => true,
            (Primitive::Int16(a), Type::Int128) => true,

            (Primitive::Int32(a), Type::Int8) => true,
            (Primitive::Int32(a), Type::Int16) => true,
            (Primitive::Int32(a), Type::Int32) => true,
            (Primitive::Int32(a), Type::Int64) => true,
            (Primitive::Int32(a), Type::Int) => true,
            (Primitive::Int32(a), Type::Int128) => true,

            (Primitive::Int64(a), Type::Int8) => true,
            (Primitive::Int64(a), Type::Int16) => true,
            (Primitive::Int64(a), Type::Int32) => true,
            (Primitive::Int64(a), Type::Int64) => true,
            (Primitive::Int64(a), Type::Int) => true,
            (Primitive::Int64(a), Type::Int128) => true,

            (Primitive::Int(a), Type::Int8) => true,
            (Primitive::Int(a), Type::Int16) => true,
            (Primitive::Int(a), Type::Int32) => true,
            (Primitive::Int(a), Type::Int64) => true,
            (Primitive::Int(a), Type::Int) => true,
            (Primitive::Int(a), Type::Int128) => true,

            (Primitive::Int128(a), Type::Int8) => true,
            (Primitive::Int128(a), Type::Int16) => true,
            (Primitive::Int128(a), Type::Int32) => true,
            (Primitive::Int128(a), Type::Int64) => true,
            (Primitive::Int128(a), Type::Int) => true,
            (Primitive::Int128(a), Type::Int128) => true,

            (Primitive::UInt8(a), Type::UInt8) => true,
            (Primitive::UInt8(a), Type::UInt16) => true,
            (Primitive::UInt8(a), Type::UInt32) => true,
            (Primitive::UInt8(a), Type::UInt64) => true,
            (Primitive::UInt8(a), Type::UInt) => true,
            (Primitive::UInt8(a), Type::UInt128) => true,

            (Primitive::UInt16(a), Type::UInt8) => true,
            (Primitive::UInt16(a), Type::UInt16) => true,
            (Primitive::UInt16(a), Type::UInt32) => true,
            (Primitive::UInt16(a), Type::UInt64) => true,
            (Primitive::UInt16(a), Type::UInt) => true,
            (Primitive::UInt16(a), Type::UInt128) => true,

            (Primitive::UInt32(a), Type::UInt8) => true,
            (Primitive::UInt32(a), Type::UInt16) => true,
            (Primitive::UInt32(a), Type::UInt32) => true,
            (Primitive::UInt32(a), Type::UInt64) => true,
            (Primitive::UInt32(a), Type::UInt) => true,
            (Primitive::UInt32(a), Type::UInt128) => true,

            (Primitive::UInt64(a), Type::UInt8) => true,
            (Primitive::UInt64(a), Type::UInt16) => true,
            (Primitive::UInt64(a), Type::UInt32) => true,
            (Primitive::UInt64(a), Type::UInt64) => true,
            (Primitive::UInt64(a), Type::UInt) => true,
            (Primitive::UInt64(a), Type::UInt128) => true,

            (Primitive::UInt(a), Type::UInt8) => true,
            (Primitive::UInt(a), Type::UInt16) => true,
            (Primitive::UInt(a), Type::UInt32) => true,
            (Primitive::UInt(a), Type::UInt64) => true,
            (Primitive::UInt(a), Type::UInt) => true,
            (Primitive::UInt(a), Type::UInt128) => true,

            (Primitive::UInt128(a), Type::UInt8) => true,
            (Primitive::UInt128(a), Type::UInt16) => true,
            (Primitive::UInt128(a), Type::UInt32) => true,
            (Primitive::UInt128(a), Type::UInt64) => true,
            (Primitive::UInt128(a), Type::UInt) => true,
            (Primitive::UInt128(a), Type::UInt128) => true,

            (Primitive::Float64(a), Type::Float32) => true,
            (Primitive::Float64(a), Type::Float64) => true,

            (Primitive::Float32(a), Type::Float32) => true,
            (Primitive::Float32(a), Type::Float64) => true,

            (Primitive::String(s), Type::String) => true,
            _ => false,
        }
    }

    pub fn is_of_type(&self, target_type: &Type) -> bool {
        match (self, target_type) {
            (Primitive::Int8(_), Type::Int8) => true,
            (Primitive::Int16(_), Type::Int16) => true,
            (Primitive::Int32(_), Type::Int32) => true,
            (Primitive::Int64(_), Type::Int64) => true,
            (Primitive::Int(_), Type::Int) => true,
            (Primitive::Int128(_), Type::Int128) => true,

            (Primitive::UInt8(_), Type::UInt8) => true,
            (Primitive::UInt16(_), Type::UInt16) => true,
            (Primitive::UInt32(_), Type::UInt32) => true,
            (Primitive::UInt64(_), Type::UInt64) => true,
            (Primitive::UInt(_), Type::UInt) => true,
            (Primitive::UInt128(_), Type::UInt128) => true,

            (Primitive::Float32(_), Type::Float32) => true,
            (Primitive::Float64(_), Type::Float64) => true,

            (Primitive::Bool(_), Type::Bool) => true,
            (Primitive::Char(_), Type::Char) => true,

            (Primitive::String(_), Type::String) => true,
            _ => false,
        }
    }

    pub fn get_type(&self) -> Type {
        match self {
            Primitive::Int8(_) => Type::Int8,
            Primitive::Int16(_) => Type::Int16,
            Primitive::Int32(_) => Type::Int32,
            Primitive::Int64(_) => Type::Int64,
            Primitive::Int(_) => Type::Int,
            Primitive::Int128(_) => Type::Int128,

            Primitive::UInt8(_) => Type::UInt8,
            Primitive::UInt16(_) => Type::UInt16,
            Primitive::UInt32(_) => Type::UInt32,
            Primitive::UInt64(_) => Type::UInt64,
            Primitive::UInt(_) => Type::UInt,
            Primitive::UInt128(_) => Type::UInt128,

            Primitive::Float32(_) => Type::Float32,
            Primitive::Float64(_) => Type::Float64,

            Primitive::Bool(_) => Type::Bool,
            Primitive::Char(_) => Type::Char,

            Primitive::String(_) => Type::String,
            Primitive::None => Type::None,
        }
    }

    fn add(p1: Primitive, p2: Primitive) -> Primitive {
        match (p1, p2) {
            (Primitive::Int8(a), Primitive::Int8(b)) => Primitive::Int8(a + b),
            (Primitive::Int16(a), Primitive::Int16(b)) => Primitive::Int16(a + b),
            (Primitive::Int32(a), Primitive::Int32(b)) => Primitive::Int32(a + b),
            (Primitive::Int64(a), Primitive::Int64(b)) => Primitive::Int64(a + b),
            (Primitive::Int(a), Primitive::Int(b)) => Primitive::Int(a + b),
            (Primitive::Int128(a), Primitive::Int128(b)) => Primitive::Int128(a + b),

            (Primitive::UInt8(a), Primitive::UInt8(b)) => Primitive::UInt8(a + b),
            (Primitive::UInt16(a), Primitive::UInt16(b)) => Primitive::UInt16(a + b),
            (Primitive::UInt32(a), Primitive::UInt32(b)) => Primitive::UInt32(a + b),
            (Primitive::UInt64(a), Primitive::UInt64(b)) => Primitive::UInt64(a + b),
            (Primitive::UInt(a), Primitive::UInt(b)) => Primitive::UInt(a + b),
            (Primitive::UInt128(a), Primitive::UInt128(b)) => Primitive::UInt128(a + b),

            (Primitive::Float32(a), Primitive::Float32(b)) => Primitive::Float32(a + b),
            (Primitive::Float64(a), Primitive::Float64(b)) => Primitive::Float64(a + b),
            _ => panic!(format!(
                "Cannot add {:?} to {:?}: Incompatible Types",
                p1, p2
            )),
        }
    }

    fn sub(p1: Primitive, p2: Primitive) -> Primitive {
        match (p1, p2) {
            (Primitive::Int8(a), Primitive::Int8(b)) => Primitive::Int8(a - b),
            (Primitive::Int16(a), Primitive::Int16(b)) => Primitive::Int16(a - b),
            (Primitive::Int32(a), Primitive::Int32(b)) => Primitive::Int32(a - b),
            (Primitive::Int64(a), Primitive::Int64(b)) => Primitive::Int64(a - b),
            (Primitive::Int(a), Primitive::Int(b)) => Primitive::Int(a - b),
            (Primitive::Int128(a), Primitive::Int128(b)) => Primitive::Int128(a - b),

            (Primitive::UInt8(a), Primitive::UInt8(b)) => Primitive::UInt8(a - b),
            (Primitive::UInt16(a), Primitive::UInt16(b)) => Primitive::UInt16(a - b),
            (Primitive::UInt32(a), Primitive::UInt32(b)) => Primitive::UInt32(a - b),
            (Primitive::UInt64(a), Primitive::UInt64(b)) => Primitive::UInt64(a - b),
            (Primitive::UInt(a), Primitive::UInt(b)) => Primitive::UInt(a - b),
            (Primitive::UInt128(a), Primitive::UInt128(b)) => Primitive::UInt128(a - b),

            (Primitive::Float32(a), Primitive::Float32(b)) => Primitive::Float32(a - b),
            (Primitive::Float64(a), Primitive::Float64(b)) => Primitive::Float64(a - b),
            _ => panic!(format!(
                "Cannot add {:?} to {:?}: Incompatible Types",
                p1, p2
            )),
        }
    }

    fn mul(p1: Primitive, p2: Primitive) -> Primitive {
        match (p1, p2) {
            (Primitive::Int8(a), Primitive::Int8(b)) => Primitive::Int8(a * b),
            (Primitive::Int16(a), Primitive::Int16(b)) => Primitive::Int16(a * b),
            (Primitive::Int32(a), Primitive::Int32(b)) => Primitive::Int32(a * b),
            (Primitive::Int64(a), Primitive::Int64(b)) => Primitive::Int64(a * b),
            (Primitive::Int(a), Primitive::Int(b)) => Primitive::Int(a * b),
            (Primitive::Int128(a), Primitive::Int128(b)) => Primitive::Int128(a * b),

            (Primitive::UInt8(a), Primitive::UInt8(b)) => Primitive::UInt8(a * b),
            (Primitive::UInt16(a), Primitive::UInt16(b)) => Primitive::UInt16(a * b),
            (Primitive::UInt32(a), Primitive::UInt32(b)) => Primitive::UInt32(a * b),
            (Primitive::UInt64(a), Primitive::UInt64(b)) => Primitive::UInt64(a * b),
            (Primitive::UInt(a), Primitive::UInt(b)) => Primitive::UInt(a * b),
            (Primitive::UInt128(a), Primitive::UInt128(b)) => Primitive::UInt128(a * b),

            (Primitive::Float32(a), Primitive::Float32(b)) => Primitive::Float32(a * b),
            (Primitive::Float64(a), Primitive::Float64(b)) => Primitive::Float64(a * b),
            _ => panic!(format!(
                "Cannot add {:?} to {:?}: Incompatible Types",
                p1, p2
            )),
        }
    }

    fn div(p1: Primitive, p2: Primitive) -> Primitive {
        match (p1, p2) {
            (Primitive::Int8(a), Primitive::Int8(b)) => Primitive::Int8(a / b),
            (Primitive::Int16(a), Primitive::Int16(b)) => Primitive::Int16(a / b),
            (Primitive::Int32(a), Primitive::Int32(b)) => Primitive::Int32(a / b),
            (Primitive::Int64(a), Primitive::Int64(b)) => Primitive::Int64(a / b),
            (Primitive::Int(a), Primitive::Int(b)) => Primitive::Int(a / b),
            (Primitive::Int128(a), Primitive::Int128(b)) => Primitive::Int128(a / b),

            (Primitive::UInt8(a), Primitive::UInt8(b)) => Primitive::UInt8(a / b),
            (Primitive::UInt16(a), Primitive::UInt16(b)) => Primitive::UInt16(a / b),
            (Primitive::UInt32(a), Primitive::UInt32(b)) => Primitive::UInt32(a / b),
            (Primitive::UInt64(a), Primitive::UInt64(b)) => Primitive::UInt64(a / b),
            (Primitive::UInt(a), Primitive::UInt(b)) => Primitive::UInt(a / b),
            (Primitive::UInt128(a), Primitive::UInt128(b)) => Primitive::UInt128(a / b),

            (Primitive::Float32(a), Primitive::Float32(b)) => Primitive::Float32(a / b),
            (Primitive::Float64(a), Primitive::Float64(b)) => Primitive::Float64(a / b),
            _ => panic!(format!(
                "Cannot add {:?} to {:?}: Incompatible Types",
                p1, p2
            )),
        }
    }

    fn pow(p1: Primitive, p2: Primitive) -> Primitive {
        match (p1, p2) {
            (Primitive::Int8(a), Primitive::UInt32(b)) => Primitive::Int8(i8::pow(a, b)),
            (Primitive::Int16(a), Primitive::UInt32(b)) => Primitive::Int16(i16::pow(a, b)),
            (Primitive::Int32(a), Primitive::UInt32(b)) => Primitive::Int32(i32::pow(a, b)),
            (Primitive::Int64(a), Primitive::UInt32(b)) => Primitive::Int64(i64::pow(a, b)),
            (Primitive::Int(a), Primitive::UInt32(b)) => Primitive::Int(isize::pow(a, b)),
            (Primitive::Int128(a), Primitive::UInt32(b)) => Primitive::Int128(i128::pow(a, b)),

            (Primitive::UInt8(a), Primitive::UInt32(b)) => Primitive::UInt8(u8::pow(a, b)),
            (Primitive::UInt16(a), Primitive::UInt32(b)) => Primitive::UInt16(u16::pow(a, b)),
            (Primitive::UInt32(a), Primitive::UInt32(b)) => Primitive::UInt32(u32::pow(a, b)),
            (Primitive::UInt64(a), Primitive::UInt32(b)) => Primitive::UInt64(u64::pow(a, b)),
            (Primitive::UInt(a), Primitive::UInt32(b)) => Primitive::UInt(usize::pow(a, b)),
            (Primitive::UInt128(a), Primitive::UInt32(b)) => Primitive::UInt128(u128::pow(a, b)),

            (Primitive::Float32(a), Primitive::Float32(b)) => Primitive::Float32(a / b),
            (Primitive::Float64(a), Primitive::Float64(b)) => Primitive::Float64(a / b),
            _ => panic!(format!(
                "Cannot add {:?} to {:?}: Incompatible Types",
                p1, p2
            )),
        }
    }

    fn rem(p1: Primitive, p2: Primitive) -> Primitive {
        match (p1, p2) {
            (Primitive::Int8(a), Primitive::Int8(b)) => Primitive::Int8(i8::rem_euclid(a, b)),
            (Primitive::Int16(a), Primitive::Int16(b)) => Primitive::Int16(i16::rem_euclid(a, b)),
            (Primitive::Int32(a), Primitive::Int32(b)) => Primitive::Int32(i32::rem_euclid(a, b)),
            (Primitive::Int64(a), Primitive::Int64(b)) => Primitive::Int64(i64::rem_euclid(a, b)),
            (Primitive::Int(a), Primitive::Int(b)) => Primitive::Int(isize::rem_euclid(a, b)),
            (Primitive::Int128(a), Primitive::Int128(b)) => {
                Primitive::Int128(i128::rem_euclid(a, b))
            }

            (Primitive::UInt8(a), Primitive::UInt8(b)) => Primitive::UInt8(u8::rem_euclid(a, b)),
            (Primitive::UInt16(a), Primitive::UInt16(b)) => {
                Primitive::UInt16(u16::rem_euclid(a, b))
            }
            (Primitive::UInt32(a), Primitive::UInt32(b)) => {
                Primitive::UInt32(u32::rem_euclid(a, b))
            }
            (Primitive::UInt64(a), Primitive::UInt64(b)) => {
                Primitive::UInt64(u64::rem_euclid(a, b))
            }
            (Primitive::UInt(a), Primitive::UInt(b)) => Primitive::UInt(usize::rem_euclid(a, b)),
            (Primitive::UInt128(a), Primitive::UInt128(b)) => {
                Primitive::UInt128(u128::rem_euclid(a, b))
            }

            _ => panic!(format!(
                "Cannot add {:?} to {:?}: Incompatible Types",
                p1, p2
            )),
        }
    }

    fn less_than(p1: Primitive, p2: Primitive) -> bool {
        match (p1, p2) {
            (Primitive::Int8(a), Primitive::Int8(b)) => a < b,
            (Primitive::Int16(a), Primitive::Int16(b)) => a < b,
            (Primitive::Int32(a), Primitive::Int32(b)) => a < b,
            (Primitive::Int64(a), Primitive::Int64(b)) => a < b,
            (Primitive::Int(a), Primitive::Int(b)) => a < b,
            (Primitive::Int128(a), Primitive::Int128(b)) => a < b,

            (Primitive::UInt8(a), Primitive::UInt8(b)) => a < b,
            (Primitive::UInt16(a), Primitive::UInt16(b)) => a < b,
            (Primitive::UInt32(a), Primitive::UInt32(b)) => a < b,
            (Primitive::UInt64(a), Primitive::UInt64(b)) => a < b,
            (Primitive::UInt(a), Primitive::UInt(b)) => a < b,
            (Primitive::UInt128(a), Primitive::UInt128(b)) => a < b,

            (Primitive::Float32(a), Primitive::Float32(b)) => a < b,
            (Primitive::Float64(a), Primitive::Float64(b)) => a < b,
            _ => panic!(format!(
                "Cannot add {:?} to {:?}: Incompatible Types",
                p1, p2
            )),
        }
    }

    fn less_than_equal(p1: Primitive, p2: Primitive) -> bool {
        match (p1, p2) {
            (Primitive::Int8(a), Primitive::Int8(b)) => a <= b,
            (Primitive::Int16(a), Primitive::Int16(b)) => a <= b,
            (Primitive::Int32(a), Primitive::Int32(b)) => a <= b,
            (Primitive::Int64(a), Primitive::Int64(b)) => a <= b,
            (Primitive::Int(a), Primitive::Int(b)) => a <= b,
            (Primitive::Int128(a), Primitive::Int128(b)) => a <= b,

            (Primitive::UInt8(a), Primitive::UInt8(b)) => a <= b,
            (Primitive::UInt16(a), Primitive::UInt16(b)) => a <= b,
            (Primitive::UInt32(a), Primitive::UInt32(b)) => a <= b,
            (Primitive::UInt64(a), Primitive::UInt64(b)) => a <= b,
            (Primitive::UInt(a), Primitive::UInt(b)) => a <= b,
            (Primitive::UInt128(a), Primitive::UInt128(b)) => a <= b,

            (Primitive::Float32(a), Primitive::Float32(b)) => a <= b,
            (Primitive::Float64(a), Primitive::Float64(b)) => a <= b,
            _ => panic!(format!(
                "Cannot add {:?} to {:?}: Incompatible Types",
                p1, p2
            )),
        }
    }

    fn greater_than(p1: Primitive, p2: Primitive) -> bool {
        match (p1, p2) {
            (Primitive::Int8(a), Primitive::Int8(b)) => a > b,
            (Primitive::Int16(a), Primitive::Int16(b)) => a > b,
            (Primitive::Int32(a), Primitive::Int32(b)) => a > b,
            (Primitive::Int64(a), Primitive::Int64(b)) => a > b,
            (Primitive::Int(a), Primitive::Int(b)) => a > b,
            (Primitive::Int128(a), Primitive::Int128(b)) => a > b,

            (Primitive::UInt8(a), Primitive::UInt8(b)) => a > b,
            (Primitive::UInt16(a), Primitive::UInt16(b)) => a > b,
            (Primitive::UInt32(a), Primitive::UInt32(b)) => a > b,
            (Primitive::UInt64(a), Primitive::UInt64(b)) => a > b,
            (Primitive::UInt(a), Primitive::UInt(b)) => a > b,
            (Primitive::UInt128(a), Primitive::UInt128(b)) => a > b,

            (Primitive::Float32(a), Primitive::Float32(b)) => a > b,
            (Primitive::Float64(a), Primitive::Float64(b)) => a > b,
            _ => panic!(format!(
                "Cannot add {:?} to {:?}: Incompatible Types",
                p1, p2
            )),
        }
    }

    fn greater_than_equal(p1: Primitive, p2: Primitive) -> bool {
        match (p1, p2) {
            (Primitive::Int8(a), Primitive::Int8(b)) => a >= b,
            (Primitive::Int16(a), Primitive::Int16(b)) => a >= b,
            (Primitive::Int32(a), Primitive::Int32(b)) => a >= b,
            (Primitive::Int64(a), Primitive::Int64(b)) => a >= b,
            (Primitive::Int(a), Primitive::Int(b)) => a >= b,
            (Primitive::Int128(a), Primitive::Int128(b)) => a >= b,

            (Primitive::UInt8(a), Primitive::UInt8(b)) => a >= b,
            (Primitive::UInt16(a), Primitive::UInt16(b)) => a >= b,
            (Primitive::UInt32(a), Primitive::UInt32(b)) => a >= b,
            (Primitive::UInt64(a), Primitive::UInt64(b)) => a >= b,
            (Primitive::UInt(a), Primitive::UInt(b)) => a >= b,
            (Primitive::UInt128(a), Primitive::UInt128(b)) => a >= b,

            (Primitive::Float32(a), Primitive::Float32(b)) => a >= b,
            (Primitive::Float64(a), Primitive::Float64(b)) => a >= b,
            _ => panic!(format!(
                "Cannot add {:?} to {:?}: Incompatible Types",
                p1, p2
            )),
        }
    }

    fn equal(p1: Primitive, p2: Primitive) -> bool {
        match (p1, p2) {
            (Primitive::Int8(a), Primitive::Int8(b)) => a == b,
            (Primitive::Int16(a), Primitive::Int16(b)) => a == b,
            (Primitive::Int32(a), Primitive::Int32(b)) => a == b,
            (Primitive::Int64(a), Primitive::Int64(b)) => a == b,
            (Primitive::Int(a), Primitive::Int(b)) => a == b,
            (Primitive::Int128(a), Primitive::Int128(b)) => a == b,

            (Primitive::UInt8(a), Primitive::UInt8(b)) => a == b,
            (Primitive::UInt16(a), Primitive::UInt16(b)) => a == b,
            (Primitive::UInt32(a), Primitive::UInt32(b)) => a == b,
            (Primitive::UInt64(a), Primitive::UInt64(b)) => a == b,
            (Primitive::UInt(a), Primitive::UInt(b)) => a == b,
            (Primitive::UInt128(a), Primitive::UInt128(b)) => a == b,

            (Primitive::Float32(a), Primitive::Float32(b)) => a == b,
            (Primitive::Float64(a), Primitive::Float64(b)) => a == b,
            _ => panic!(format!(
                "Cannot add {:?} to {:?}: Incompatible Types",
                p1, p2
            )),
        }
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Primitive::Int8(a) => write!(f, "{}", a),
            Primitive::Int16(a) => write!(f, "{}", a),
            Primitive::Int32(a) => write!(f, "{}", a),
            Primitive::Int64(a) => write!(f, "{}", a),
            Primitive::Int(a) => write!(f, "{}", a),
            Primitive::Int128(a) => write!(f, "{}", a),

            Primitive::UInt8(a) => write!(f, "{}", a),
            Primitive::UInt16(a) => write!(f, "{}", a),
            Primitive::UInt32(a) => write!(f, "{}", a),
            Primitive::UInt64(a) => write!(f, "{}", a),
            Primitive::UInt(a) => write!(f, "{}", a),
            Primitive::UInt128(a) => write!(f, "{}", a),

            // Floating PoInt Primitives
            Primitive::Float32(a) => write!(f, "{:?}", a),
            Primitive::Float64(a) => write!(f, "{:?}", a),

            // Boolean Primitive
            Primitive::Bool(a) => {
                if *a {
                    write!(f, "true")?;
                } else {
                    write!(f, "false")?;
                }
                Ok(())
            }

            // Character Primitive
            Primitive::Char(a) => write!(f, "{:?}", a),

            // String Primitive
            Primitive::String(u) => write!(f, "{:?}", get_string(*u)),

            // NoneType
            Primitive::None => write!(f, "none"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Value {
    pub inner: Primitive,
}

impl Value {
    pub fn can_cast_to(&self, target_type: &Type) -> bool {
        self.inner.can_inference_to(target_type)
    }

    pub fn try_cast_to(&mut self, target_type: &Type) -> Result<(), ()> {
        self.inner = self.inner.try_cast_to(target_type)?;
        Ok(())
    }

    pub fn is_of_type(&self, target_type: &Type) -> bool {
        self.inner.is_of_type(target_type)
    }

    pub fn get_type(&self) -> Type {
        self.inner.get_type()
    }

    fn add(left: Value, right: Value) -> Value {
        Value::from(Primitive::add(left.inner, right.inner))
    }

    fn sub(left: Value, right: Value) -> Value {
        Value::from(Primitive::sub(left.inner, right.inner))
    }

    fn mul(left: Value, right: Value) -> Value {
        Value::from(Primitive::mul(left.inner, right.inner))
    }

    fn div(left: Value, right: Value) -> Value {
        Value::from(Primitive::div(left.inner, right.inner))
    }

    fn pow(left: Value, right: Value) -> Value {
        Value::from(Primitive::pow(left.inner, right.inner))
    }

    fn rem(left: Value, right: Value) -> Value {
        Value::from(Primitive::rem(left.inner, right.inner))
    }

    fn less_than(left: Value, right: Value) -> bool {
        Primitive::less_than(left.inner, right.inner)
    }

    fn less_than_equal(left: Value, right: Value) -> bool {
        Primitive::less_than_equal(left.inner, right.inner)
    }

    fn greater_than(left: Value, right: Value) -> bool {
        Primitive::greater_than(left.inner, right.inner)
    }

    fn greater_than_equal(left: Value, right: Value) -> bool {
        Primitive::greater_than_equal(left.inner, right.inner)
    }

    fn equal(left: Value, right: Value) -> bool {
        Primitive::equal(left.inner, right.inner)
    }

    fn not_equal(left: Value, right: Value) -> bool {
        !Primitive::equal(left.inner, right.inner)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.inner)
    }
}

impl From<Primitive> for Value {
    fn from(other: Primitive) -> Self {
        Self { inner: other }
    }
}

pub fn compute_binary(left: Value, op: BinOp, right: Value) -> Value {
    match op {
        BinOp::Plus => Value::add(left, right),
        BinOp::Minus => Value::sub(left, right),
        BinOp::Star => Value::mul(left, right),
        BinOp::Slash => Value::div(left, right),
        BinOp::Carat => Value::pow(left, right),
        BinOp::Modulo => Value::rem(left, right),
        _ => unimplemented!(),
    }
}

pub fn compute_logical(left: Value, op: RelOp, right: Value) -> Value {
    let res = match op {
        RelOp::Less => Value::less_than(left, right),
        RelOp::LessEqual => Value::less_than_equal(left, right),
        RelOp::Greater => Value::greater_than(left, right),
        RelOp::GreaterEqual => Value::greater_than_equal(left, right),
        RelOp::EqualEqual => Value::equal(left, right),
        RelOp::NotEqual => Value::not_equal(left, right),
        _ => unimplemented!(),
    };
    Value::from(Primitive::Bool(res))
}