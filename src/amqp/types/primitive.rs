use std::{collections::HashMap, hash::Hash}; 

use super::constructor::Constructor;

#[derive(Debug, Clone)]
pub struct InnerFloat {
    pub value: f32,
}

#[derive(Debug, Clone)]
pub struct InnerDouble {
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct InnerMap {
    pub value: HashMap<Constructor, Constructor>,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum Primitive {
    // Fixed width
    Null,
    Boolean(bool),
    UByte(u8),
    UShort(u16),
    UInt(u32),
    ULong(u64),
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(InnerFloat),
    Double(InnerDouble),
    Decimal32([u8; 4]),
    Decimal64([u8; 8]),
    Decimal128([u8; 16]),
    Char([u8; 4]),
    Timestamp(i64),
    UUID([u8; 16]),

    // Variable width
    Binary(Vec<u8>),
    String(String),
    Symbol(Vec<u8>),

    // Compound
    EmptyList,
    List(Vec<Constructor>),
    Map(InnerMap),

    // Arrays
    Array(Vec<Constructor>),
}

impl PartialEq for InnerFloat {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialEq for InnerDouble {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialEq for InnerMap {
    fn eq(&self, other: &Self) -> bool {
        if self.value.len() != other.value.len() {
            return false;
        }
        for (elt_key, elt_val) in self.value.iter() {
            match other.value.get(elt_key) {
                Some(other_elt_val) => {
                    if elt_val != other_elt_val {
                        return false;
                    }
                }
                None => {
                    return false;
                }
            }
        }
        return true;
    }
}

impl Eq for InnerFloat {}
impl Eq for InnerDouble {}
impl Eq for InnerMap {}

impl Hash for InnerFloat {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let repr = self.value.to_string();
        repr.hash(state);
    }
}

impl Hash for InnerDouble {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let repr = self.value.to_string();
        repr.hash(state);
    }
}

impl Hash for InnerMap {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let len = self.value.len();
        len.hash(state);
        for (elt_key, elt_val) in self.value.iter() {
            let elt_key = (*elt_key).clone();
            let elt_val = (*elt_val).clone();
            elt_key.hash(state);
            elt_val.hash(state);
        }
    }
}
