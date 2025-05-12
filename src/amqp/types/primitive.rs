use super::constructor::Constructor;

pub enum Primitive<'a> {
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
    Float(f32),
    Double(f64),
    Decimal32([u8; 4]),
    Decimal64([u8; 8]),
    Decimal128([u8; 16]),
    Char([u8; 4]),
    Timestamp(i64),
    UUID([u8; 16]),

    // Variable width
    OneByteBinary(u8, &'a [u8]),
    FourByteBinary(u32, &'a [u8]),
    OneByteString(u8, &'a [u8]),
    FourByteString(u32, &'a [u8]),
    OneByteSymbol(u8, &'a [u8]),
    FourByteSymbol(u32, &'a [u8]),

    // Compound
    EmptyList,
    OneByteList(u8, u8, &'a [u8]),
    FourByteList(u32, u32, &'a [u8]),
    OneByteMap(u8, &'a [u8]),
    FourByteMap(u32, &'a [u8]),

    // Arrays
    OneByteArray(u8, u8, Option<Box<Constructor<'a>>>, &'a [u8]),
    FourByteArray(u32, u32, Option<Box<Constructor<'a>>>, &'a [u8]),
}
