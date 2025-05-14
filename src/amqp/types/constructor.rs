use std::io::Read;

use super::primitive::Primitive;

use super::format_code::FormatCode;

pub enum Constructor {
    PrimitiveType(Primitive),
    DescribedType(Box<Constructor>, Primitive),
}

impl Constructor {
    pub fn new(code: FormatCode, buf_reader: &mut impl Read) -> Result<Self, &'static str> {
        match code {
            FormatCode::NonPrimitive => {
                let mut read_buf = [0u8; 2];
                buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                let descriptor_code = FormatCode::try_from(u16::from_be_bytes(read_buf))?;
                let descriptor = Constructor::new(descriptor_code, buf_reader)?;
                buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                let primitive_code = FormatCode::try_from(u16::from_be_bytes(read_buf))?;
                match primitive_code {
                    FormatCode::NonPrimitive => {
                        Err("Non-primitive used as a described constructor primitive")
                    }
                    _ => {
                        let primitive = Constructor::new(primitive_code, buf_reader)?;
                        match primitive {
                            Constructor::PrimitiveType(primitive) => {
                                Ok(Self::DescribedType(Box::new(descriptor), primitive))
                            }
                            _ => Err("Non-primitive used as a described constructor primitive"),
                        }
                    }
                }
            }
            _ => Ok(Self::PrimitiveType(match code {
                FormatCode::NonPrimitive => unreachable!(),
                FormatCode::Null => Primitive::Null,
                FormatCode::Boolean => {
                    let mut buf = [0u8; 1];
                    buf_reader.read_exact(&mut buf).unwrap_or_else(|_| {});
                    Primitive::Boolean(buf[0] != 0)
                }
                FormatCode::BooleanTrue => Primitive::Boolean(true),
                FormatCode::BooleanFalse => Primitive::Boolean(false),
                FormatCode::Ubyte => {
                    let mut buf = [0u8; 1];
                    buf_reader.read_exact(&mut buf).unwrap_or_else(|_| {});
                    Primitive::UByte(buf[0])
                }
                FormatCode::Ushort => {
                    let mut buf = [0u8; 2];
                    buf_reader.read_exact(&mut buf).unwrap_or_else(|_| {});
                    Primitive::UShort(u16::from_be_bytes(buf))
                }
                FormatCode::Uint => {
                    let mut buf = [0u8; 4];
                    buf_reader.read_exact(&mut buf).unwrap_or_else(|_| {});
                    Primitive::UInt(u32::from_be_bytes(buf))
                }
                FormatCode::Smalluint => {
                    let mut buf = [0u8; 4];
                    buf_reader.read_exact(&mut buf[3..]).unwrap_or_else(|_| {});
                    Primitive::UInt(u32::from_be_bytes(buf))
                }
                FormatCode::Uint0 => Primitive::UInt(0),
                FormatCode::Ulong => {
                    let mut buf = [0u8; 8];
                    buf_reader.read_exact(&mut buf).unwrap_or_else(|_| {});
                    Primitive::ULong(u64::from_be_bytes(buf))
                }
                FormatCode::Smallulong => {
                    let mut buf = [0u8; 8];
                    buf_reader.read_exact(&mut buf[7..]).unwrap_or_else(|_| {});
                    Primitive::ULong(u64::from_be_bytes(buf))
                }
                FormatCode::Ulong0 => Primitive::ULong(0),
                FormatCode::Byte => {
                    let mut buf = [0u8; 1];
                    buf_reader.read_exact(&mut buf).unwrap_or_else(|_| {});
                    Primitive::Byte(i8::from_be_bytes(buf))
                }
                FormatCode::Short => {
                    let mut buf = [0u8; 2];
                    buf_reader.read_exact(&mut buf).unwrap_or_else(|_| {});
                    Primitive::Short(i16::from_be_bytes(buf))
                }
                FormatCode::Int => {
                    let mut buf = [0u8; 4];
                    buf_reader.read_exact(&mut buf).unwrap_or_else(|_| {});
                    Primitive::Int(i32::from_be_bytes(buf))
                }
                FormatCode::Smallint => {
                    let mut buf = [0u8; 4];
                    buf_reader.read_exact(&mut buf[3..]).unwrap_or_else(|_| {});
                    Primitive::Int(i32::from_be_bytes(buf))
                }
                FormatCode::Long => {
                    let mut buf = [0u8; 8];
                    buf_reader.read_exact(&mut buf).unwrap_or_else(|_| {});
                    Primitive::Long(i64::from_be_bytes(buf))
                }
                FormatCode::Smalllong => {
                    let mut buf = [0u8; 8];
                    buf_reader.read_exact(&mut buf[7..]).unwrap_or_else(|_| {});
                    Primitive::Long(i64::from_be_bytes(buf))
                }
                FormatCode::Float => {
                    let mut buf = [0u8; 4];
                    buf_reader.read_exact(&mut buf).unwrap_or_else(|_| {});
                    Primitive::Float(f32::from_be_bytes(buf))
                }
                FormatCode::Double => {
                    let mut buf = [0u8; 8];
                    buf_reader.read_exact(&mut buf).unwrap_or_else(|_| {});
                    Primitive::Double(f64::from_be_bytes(buf))
                }
                FormatCode::Decimal32 => {
                    let mut buf = [0u8; 4];
                    buf_reader.read_exact(&mut buf).unwrap_or_else(|_| {});
                    Primitive::Decimal32(buf)
                }
                FormatCode::Decimal64 => {
                    let mut buf = [0u8; 8];
                    buf_reader.read_exact(&mut buf).unwrap_or_else(|_| {});
                    Primitive::Decimal64(buf)
                }
                FormatCode::Decimal128 => {
                    let mut buf = [0u8; 16];
                    buf_reader.read_exact(&mut buf).unwrap_or_else(|_| {});
                    Primitive::Decimal128(buf)
                }
                FormatCode::Char => {
                    let mut buf = [0u8; 4];
                    buf_reader.read_exact(&mut buf).unwrap_or_else(|_| {});
                    Primitive::Char(buf)
                }
                FormatCode::Timestamp => {
                    let mut buf = [0u8; 8];
                    buf_reader.read_exact(&mut buf).unwrap_or_else(|_| {});
                    Primitive::Timestamp(i64::from_be_bytes(buf))
                }
                FormatCode::Uuid => {
                    let mut buf = [0u8; 16];
                    buf_reader.read_exact(&mut buf).unwrap_or_else(|_| {});
                    Primitive::UUID(buf)
                }
                FormatCode::OneByteBinary => {
                    let mut read_buf = [0u8; 1];
                    buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                    let len = read_buf[0];

                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                        buf.push(read_buf[0]);
                    }
                    Primitive::OneByteBinary(len, buf)
                }
                FormatCode::FourByteBinary => {
                    let mut read_buf = [0u8; 4];
                    buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                    let len = u32::from_be_bytes(read_buf);

                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                        buf.push(read_buf[0]);
                    }
                    Primitive::FourByteBinary(len, buf)
                }
                FormatCode::OneByteString => {
                    let mut read_buf = [0u8; 1];
                    buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                    let len = read_buf[0];

                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                        buf.push(read_buf[0]);
                    }
                    Primitive::OneByteString(len, buf)
                }
                FormatCode::FourByteString => {
                    let mut read_buf = [0u8; 4];
                    buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                    let len = u32::from_be_bytes(read_buf);

                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                        buf.push(read_buf[0]);
                    }
                    Primitive::FourByteString(len, buf)
                }
                FormatCode::OneByteSymbol => {
                    let mut read_buf = [0u8; 1];
                    buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                    let len = read_buf[0];

                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                        buf.push(read_buf[0]);
                    }
                    Primitive::OneByteSymbol(len, buf)
                }
                FormatCode::FourByteSymbol => {
                    let mut read_buf = [0u8; 4];
                    buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                    let len = u32::from_be_bytes(read_buf);

                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                        buf.push(read_buf[0]);
                    }
                    Primitive::FourByteSymbol(len, buf)
                }
                FormatCode::List0 => Primitive::EmptyList,
                FormatCode::List8 => {
                    let mut read_buf = [0u8; 2];
                    buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                    let size = read_buf[0];
                    let count = read_buf[1];
                    let len = (size * count) as usize;
                    let mut buf = Vec::<u8>::with_capacity(len);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                        buf.push(read_buf[0]);
                    }
                    Primitive::OneByteList(size, count, buf)
                }
                FormatCode::List32 => {
                    let mut read_buf = [0u8; 8];
                    buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                    let size =
                        u32::from_be_bytes([read_buf[0], read_buf[1], read_buf[2], read_buf[3]]);
                    let count =
                        u32::from_be_bytes([read_buf[4], read_buf[5], read_buf[6], read_buf[7]]);
                    let len = (size * count) as usize;
                    let mut buf = Vec::<u8>::with_capacity(len);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                        buf.push(read_buf[0]);
                    }
                    Primitive::FourByteList(size, count, buf)
                }
                FormatCode::Map8 => {
                    let mut read_buf = [0u8; 1];
                    buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                    let len = read_buf[0];
                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                        buf.push(read_buf[0]);
                    }
                    Primitive::OneByteMap(len, buf)
                }
                FormatCode::Map32 => {
                    let mut read_buf = [0u8; 4];
                    buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                    let len = u32::from_be_bytes(read_buf);
                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    for _ in 0..len {
                        buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                        buf.push(read_buf[0]);
                    }
                    Primitive::FourByteMap(len, buf)
                }
                FormatCode::Array8 => {
                    let mut read_buf = [0u8; 1];
                    buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                    let size = read_buf[0];
                    buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                    let count = read_buf[0];

                    let mut read_buf = [0u8; 2];
                    buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                    let constructor_code = FormatCode::try_from(u16::from_be_bytes(read_buf))?;
                    let constructor = Constructor::new(constructor_code, buf_reader);
                    match constructor {
                        Ok(constructor) => {
                            let len = (size * count) as usize;
                            let mut buf = Vec::<u8>::with_capacity(len);

                            let mut read_buf = [0u8; 1];
                            for _ in 0..len {
                                buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                                buf.push(read_buf[0]);
                            }
                            Primitive::OneByteArray(size, count, Some(Box::new(constructor)), buf)
                        }
                        Err(_) => Primitive::Null,
                    }
                }
                FormatCode::Array32 => {
                    let mut read_buf = [0u8; 4];
                    buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                    let size = u32::from_be_bytes(read_buf);
                    buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                    let count = u32::from_be_bytes(read_buf);

                    let read_buf = [0u8; 2];
                    let constructor_code = FormatCode::try_from(u16::from_be_bytes(read_buf))?;
                    let constructor = Constructor::new(constructor_code, buf_reader);
                    match constructor {
                        Ok(constructor) => {
                            let len = (size * count) as usize;
                            let mut buf = Vec::<u8>::with_capacity(len);

                            let mut read_buf = [0u8; 1];
                            for _ in 0..len {
                                buf_reader.read_exact(&mut read_buf).unwrap_or_else(|_| {});
                                buf.push(read_buf[0]);
                            }
                            Primitive::FourByteArray(size, count, Some(Box::new(constructor)), buf)
                        }
                        Err(_) => Primitive::Null,
                    }
                }
            })),
        }
    }
}
