use std::io::Read;

use super::primitive::Primitive;

use super::format_code::FormatCode;

pub type Descriptor<'a> = Vec<Primitive<'a>>;

pub enum Constructor<'a> {
    PrimitiveType(Primitive<'a>),
    DescribedType(Descriptor<'a>, Primitive<'a>),
}

impl Constructor<'_> {
    pub fn new(code: FormatCode, buf_reader: &mut impl Read) -> Self {
        match code {
            FormatCode::NonPrimitive => Self::DescribedType(vec![], Primitive::Null),
            _ => Self::PrimitiveType(match code {
                FormatCode::NonPrimitive => unreachable!(),
                FormatCode::Null => Primitive::Null,
                FormatCode::Boolean => {
                    let mut buf = [0u8; 1];
                    buf_reader.read_exact(&mut buf);
                    Primitive::Boolean(buf[0] != 0)
                }
                FormatCode::BooleanTrue => Primitive::Boolean(true),
                FormatCode::BooleanFalse => Primitive::Boolean(false),
                FormatCode::Ubyte => {
                    let mut buf = [0u8; 1];
                    buf_reader.read_exact(&mut buf);
                    Primitive::UByte(buf[0])
                }
                FormatCode::Ushort => {
                    let mut buf = [0u8; 2];
                    buf_reader.read_exact(&mut buf);
                    Primitive::UShort(u16::from_be_bytes(buf))
                }
                FormatCode::Uint => {
                    let mut buf = [0u8; 4];
                    buf_reader.read_exact(&mut buf);
                    Primitive::UInt(u32::from_be_bytes(buf))
                }
                FormatCode::Smalluint => {
                    let mut buf = [0u8; 4];
                    buf_reader.read_exact(&mut buf[3..]);
                    Primitive::UInt(u32::from_be_bytes(buf))
                }
                FormatCode::Uint0 => Primitive::UInt(0),
                FormatCode::Ulong => {
                    let mut buf = [0u8; 8];
                    buf_reader.read_exact(&mut buf);
                    Primitive::ULong(u64::from_be_bytes(buf))
                }
                FormatCode::Smallulong => {
                    let mut buf = [0u8; 8];
                    buf_reader.read_exact(&mut buf[7..]);
                    Primitive::ULong(u64::from_be_bytes(buf))
                }
                FormatCode::Ulong0 => Primitive::ULong(0),
                FormatCode::Byte => {
                    let mut buf = [0u8; 1];
                    buf_reader.read_exact(&mut buf);
                    Primitive::Byte(i8::from_be_bytes(buf))
                }
                FormatCode::Short => {
                    let mut buf = [0u8; 2];
                    buf_reader.read_exact(&mut buf);
                    Primitive::Short(i16::from_be_bytes(buf))
                }
                FormatCode::Int => {
                    let mut buf = [0u8; 4];
                    buf_reader.read_exact(&mut buf);
                    Primitive::Int(i32::from_be_bytes(buf))
                }
                FormatCode::Smallint => {
                    let mut buf = [0u8; 4];
                    buf_reader.read_exact(&mut buf[3..]);
                    Primitive::Int(i32::from_be_bytes(buf))
                }
                FormatCode::Long => {
                    let mut buf = [0u8; 8];
                    buf_reader.read_exact(&mut buf);
                    Primitive::Long(i64::from_be_bytes(buf))
                }
                FormatCode::Smalllong => {
                    let mut buf = [0u8; 8];
                    buf_reader.read_exact(&mut buf[7..]);
                    Primitive::Long(i64::from_be_bytes(buf))
                }
                FormatCode::Float => {
                    let mut buf = [0u8; 4];
                    buf_reader.read_exact(&mut buf);
                    Primitive::Float(f32::from_be_bytes(buf))
                }
                FormatCode::Double => {
                    let mut buf = [0u8; 8];
                    buf_reader.read_exact(&mut buf);
                    Primitive::Double(f64::from_be_bytes(buf))
                }
                FormatCode::Decimal32 => {
                    let mut buf = [0u8; 4];
                    buf_reader.read_exact(&mut buf);
                    Primitive::Decimal32(buf)
                }
                FormatCode::Decimal64 => {
                    let mut buf = [0u8; 8];
                    buf_reader.read_exact(&mut buf);
                    Primitive::Decimal64(buf)
                }
                FormatCode::Decimal128 => {
                    let mut buf = [0u8; 16];
                    buf_reader.read_exact(&mut buf);
                    Primitive::Decimal128(buf)
                }
                FormatCode::Char => {
                    let mut buf = [0u8; 4];
                    buf_reader.read_exact(&mut buf);
                    Primitive::Char(buf)
                }
                FormatCode::Timestamp => {
                    let mut buf = [0u8; 8];
                    buf_reader.read_exact(&mut buf);
                    Primitive::Timestamp(i64::from_be_bytes(buf))
                }
                FormatCode::Uuid => {
                    let mut buf = [0u8; 16];
                    buf_reader.read_exact(&mut buf);
                    Primitive::UUID(buf)
                }
                FormatCode::OneByteBinary => {
                    let mut read_buf = [0u8; 1];
                    buf_reader.read_exact(&mut read_buf);
                    let len = read_buf[0];

                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf);
                        buf.push(read_buf[0]);
                    }
                    Primitive::OneByteBinary(len, buf.as_slice())
                }
                FormatCode::FourByteBinary => {
                    let mut read_buf = [0u8; 4];
                    buf_reader.read_exact(&mut read_buf);
                    let len = u32::from_be_bytes(read_buf);

                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf);
                        buf.push(read_buf[0]);
                    }
                    Primitive::FourByteBinary(len, buf.as_slice())
                }
                FormatCode::OneByteString => {
                    let mut read_buf = [0u8; 1];
                    buf_reader.read_exact(&mut read_buf);
                    let len = read_buf[0];

                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf);
                        buf.push(read_buf[0]);
                    }
                    Primitive::OneByteString(len, buf.as_slice())
                }
                FormatCode::FourByteString => {
                    let mut read_buf = [0u8; 4];
                    buf_reader.read_exact(&mut read_buf);
                    let len = u32::from_be_bytes(read_buf);

                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf);
                        buf.push(read_buf[0]);
                    }
                    Primitive::FourByteString(len, buf.as_slice())
                }
                FormatCode::OneByteSymbol => {
                    let mut read_buf = [0u8; 1];
                    buf_reader.read_exact(&mut read_buf);
                    let len = read_buf[0];

                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf);
                        buf.push(read_buf[0]);
                    }
                    Primitive::OneByteSymbol(len, buf.as_slice())
                }
                FormatCode::FourByteSymbol => {
                    let mut read_buf = [0u8; 4];
                    buf_reader.read_exact(&mut read_buf);
                    let len = u32::from_be_bytes(read_buf);

                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf);
                        buf.push(read_buf[0]);
                    }
                    Primitive::FourByteSymbol(len, buf.as_slice())
                }
                FormatCode::List0 => Primitive::EmptyList,
                FormatCode::List8 => {
                    let mut read_buf = [0u8; 2];
                    buf_reader.read_exact(&mut read_buf);
                    let size = read_buf[0];
                    let count = read_buf[1];
                    let len = (size * count) as usize;
                    let mut buf = Vec::<u8>::with_capacity(len);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf);
                        buf.push(read_buf[0]);
                    }
                    Primitive::OneByteList(size, count, buf.as_slice())
                }
                FormatCode::List32 => {
                    let mut read_buf = [0u8; 8];
                    buf_reader.read_exact(&mut read_buf);
                    let size =
                        u32::from_be_bytes([read_buf[0], read_buf[1], read_buf[2], read_buf[3]]);
                    let count =
                        u32::from_be_bytes([read_buf[4], read_buf[5], read_buf[6], read_buf[7]]);
                    let len = (size * count) as usize;
                    let mut buf = Vec::<u8>::with_capacity(len);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf);
                        buf.push(read_buf[0]);
                    }
                    Primitive::FourByteList(size, count, buf.as_slice())
                }
                FormatCode::Map8 => {
                    let mut read_buf = [0u8; 1];
                    buf_reader.read_exact(&mut read_buf);
                    let len = read_buf[0];
                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf);
                        buf.push(read_buf[0]);
                    }
                    Primitive::OneByteMap(len, buf.as_slice())
                }
                FormatCode::Map32 => {
                    let mut read_buf = [0u8; 4];
                    buf_reader.read_exact(&mut read_buf);
                    let len = u32::from_be_bytes(read_buf);
                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf);
                        buf.push(read_buf[0]);
                    }
                    Primitive::FourByteMap(len, buf.as_slice())
                }
                FormatCode::Array8 => {
                    let mut read_buf = [0u8; 1];
                    buf_reader.read_exact(&mut read_buf);
                    let size = read_buf[0];
                    buf_reader.read_exact(&mut read_buf);
                    let count = read_buf[0];

                    let mut read_buf = [0u8; 2];
                    buf_reader.read_exact(&mut read_buf);
                    let constructor_code = FormatCode::try_from(u16::from_be_bytes(read_buf))?;
                    let constructor = Constructor::new(constructor_code, buf_reader);

                    let len = (size * count) as usize;
                    let mut buf = Vec::<u8>::with_capacity(len);

                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf);
                        buf.push(read_buf[0]);
                    }
                    Primitive::OneByteArray(
                        size,
                        count,
                        Some(Box::new(constructor)),
                        buf.as_slice(),
                    )
                }
                FormatCode::Array32 => {
                    let mut read_buf = [0u8; 4];
                    buf_reader.read_exact(&mut read_buf);
                    let size = u32::from_be_bytes(read_buf);
                    buf_reader.read_exact(&mut read_buf);
                    let count = u32::from_be_bytes(read_buf);

                    let mut read_buf = [0u8; 2];
                    let constructor_code = FormatCode::try_from(u16::from_be_bytes(read_buf))?;
                    let constructor = Constructor::new(constructor_code, buf_reader);

                    let len = (size * count) as usize;
                    let mut buf = Vec::<u8>::with_capacity(len);

                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf);
                        buf.push(read_buf[0]);
                    }
                    Primitive::FourByteArray(
                        size,
                        count,
                        Some(Box::new(constructor)),
                        buf.as_slice(),
                    )
                }
            }),
        }
    }
}
