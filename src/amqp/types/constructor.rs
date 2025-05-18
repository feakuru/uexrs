use std::collections::HashMap;
use std::pin::Pin;

use tokio::io::AsyncReadExt;

use super::format_code::FormatCode;
use super::primitive::{InnerDouble, InnerFloat, InnerMap, Primitive};

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum Constructor {
    PrimitiveType(Primitive),
    DescribedType(Pin<Box<Constructor>>, Primitive),
}

impl Constructor {
    pub async fn new(
        code: FormatCode,
        buf_reader: &mut (impl AsyncReadExt + Unpin),
    ) -> Result<Self, &'static str> {
        match code {
            FormatCode::NonPrimitive => {
                let mut read_buf = [0u8; 2];
                buf_reader
                    .read_exact(&mut read_buf)
                    .await
                    .unwrap_or_else(|_| 0);
                let descriptor_code = FormatCode::try_from(u16::from_be_bytes(read_buf))?;
                let descriptor = Box::pin(Constructor::new(descriptor_code, buf_reader)).await?;
                buf_reader
                    .read_exact(&mut read_buf)
                    .await
                    .unwrap_or_else(|_| 0);
                let primitive_code = FormatCode::try_from(u16::from_be_bytes(read_buf))?;
                match primitive_code {
                    FormatCode::NonPrimitive => {
                        Err("Non-primitive used as a described constructor primitive")
                    }
                    _ => {
                        let primitive =
                            Box::pin(Constructor::new(primitive_code, buf_reader)).await?;
                        match primitive {
                            Constructor::PrimitiveType(primitive) => {
                                Ok(Self::DescribedType(Box::pin(descriptor), primitive))
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
                    buf_reader.read_exact(&mut buf).await.unwrap_or_else(|_| 0);
                    Primitive::Boolean(buf[0] != 0)
                }
                FormatCode::BooleanTrue => Primitive::Boolean(true),
                FormatCode::BooleanFalse => Primitive::Boolean(false),
                FormatCode::Ubyte => {
                    let mut buf = [0u8; 1];
                    buf_reader.read_exact(&mut buf).await.unwrap_or_else(|_| 0);
                    Primitive::UByte(buf[0])
                }
                FormatCode::Ushort => {
                    let mut buf = [0u8; 2];
                    buf_reader.read_exact(&mut buf).await.unwrap_or_else(|_| 0);
                    Primitive::UShort(u16::from_be_bytes(buf))
                }
                FormatCode::Uint => {
                    let mut buf = [0u8; 4];
                    buf_reader.read_exact(&mut buf).await.unwrap_or_else(|_| 0);
                    Primitive::UInt(u32::from_be_bytes(buf))
                }
                FormatCode::Smalluint => {
                    let mut buf = [0u8; 4];
                    buf_reader
                        .read_exact(&mut buf[3..])
                        .await
                        .unwrap_or_else(|_| 0);
                    Primitive::UInt(u32::from_be_bytes(buf))
                }
                FormatCode::Uint0 => Primitive::UInt(0),
                FormatCode::Ulong => {
                    let mut buf = [0u8; 8];
                    buf_reader.read_exact(&mut buf).await.unwrap_or_else(|_| 0);
                    Primitive::ULong(u64::from_be_bytes(buf))
                }
                FormatCode::Smallulong => {
                    let mut buf = [0u8; 8];
                    buf_reader
                        .read_exact(&mut buf[7..])
                        .await
                        .unwrap_or_else(|_| 0);
                    Primitive::ULong(u64::from_be_bytes(buf))
                }
                FormatCode::Ulong0 => Primitive::ULong(0),
                FormatCode::Byte => {
                    let mut buf = [0u8; 1];
                    buf_reader.read_exact(&mut buf).await.unwrap_or_else(|_| 0);
                    Primitive::Byte(i8::from_be_bytes(buf))
                }
                FormatCode::Short => {
                    let mut buf = [0u8; 2];
                    buf_reader.read_exact(&mut buf).await.unwrap_or_else(|_| 0);
                    Primitive::Short(i16::from_be_bytes(buf))
                }
                FormatCode::Int => {
                    let mut buf = [0u8; 4];
                    buf_reader.read_exact(&mut buf).await.unwrap_or_else(|_| 0);
                    Primitive::Int(i32::from_be_bytes(buf))
                }
                FormatCode::Smallint => {
                    let mut buf = [0u8; 4];
                    buf_reader
                        .read_exact(&mut buf[3..])
                        .await
                        .unwrap_or_else(|_| 0);
                    Primitive::Int(i32::from_be_bytes(buf))
                }
                FormatCode::Long => {
                    let mut buf = [0u8; 8];
                    buf_reader.read_exact(&mut buf).await.unwrap_or_else(|_| 0);
                    Primitive::Long(i64::from_be_bytes(buf))
                }
                FormatCode::Smalllong => {
                    let mut buf = [0u8; 8];
                    buf_reader
                        .read_exact(&mut buf[7..])
                        .await
                        .unwrap_or_else(|_| 0);
                    Primitive::Long(i64::from_be_bytes(buf))
                }
                FormatCode::Float => {
                    let mut buf = [0u8; 4];
                    buf_reader.read_exact(&mut buf).await.unwrap_or_else(|_| 0);
                    Primitive::Float(InnerFloat {
                        value: f32::from_be_bytes(buf),
                    })
                }
                FormatCode::Double => {
                    let mut buf = [0u8; 8];
                    buf_reader.read_exact(&mut buf).await.unwrap_or_else(|_| 0);
                    Primitive::Double(InnerDouble {
                        value: f64::from_be_bytes(buf),
                    })
                }
                FormatCode::Decimal32 => {
                    let mut buf = [0u8; 4];
                    buf_reader.read_exact(&mut buf).await.unwrap_or_else(|_| 0);
                    Primitive::Decimal32(buf)
                }
                FormatCode::Decimal64 => {
                    let mut buf = [0u8; 8];
                    buf_reader.read_exact(&mut buf).await.unwrap_or_else(|_| 0);
                    Primitive::Decimal64(buf)
                }
                FormatCode::Decimal128 => {
                    let mut buf = [0u8; 16];
                    buf_reader.read_exact(&mut buf).await.unwrap_or_else(|_| 0);
                    Primitive::Decimal128(buf)
                }
                FormatCode::Char => {
                    let mut buf = [0u8; 4];
                    buf_reader.read_exact(&mut buf).await.unwrap_or_else(|_| 0);
                    Primitive::Char(buf)
                }
                FormatCode::Timestamp => {
                    let mut buf = [0u8; 8];
                    buf_reader.read_exact(&mut buf).await.unwrap_or_else(|_| 0);
                    Primitive::Timestamp(i64::from_be_bytes(buf))
                }
                FormatCode::Uuid => {
                    let mut buf = [0u8; 16];
                    buf_reader.read_exact(&mut buf).await.unwrap_or_else(|_| 0);
                    Primitive::UUID(buf)
                }
                FormatCode::OneByteBinary => {
                    let mut read_buf = [0u8; 1];
                    buf_reader
                        .read_exact(&mut read_buf)
                        .await
                        .unwrap_or_else(|_| 0);
                    let len = read_buf[0];

                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader
                            .read_exact(&mut read_buf)
                            .await
                            .unwrap_or_else(|_| 0);
                        buf.push(read_buf[0]);
                    }
                    Primitive::Binary(buf)
                }
                FormatCode::FourByteBinary => {
                    let mut read_buf = [0u8; 4];
                    buf_reader
                        .read_exact(&mut read_buf)
                        .await
                        .unwrap_or_else(|_| 0);
                    let len = u32::from_be_bytes(read_buf);

                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader
                            .read_exact(&mut read_buf)
                            .await
                            .unwrap_or_else(|_| 0);
                        buf.push(read_buf[0]);
                    }
                    Primitive::Binary(buf)
                }
                FormatCode::OneByteString => {
                    let mut read_buf = [0u8; 1];
                    buf_reader
                        .read_exact(&mut read_buf)
                        .await
                        .unwrap_or_else(|_| 0);
                    let len = read_buf[0];

                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader
                            .read_exact(&mut read_buf)
                            .await
                            .unwrap_or_else(|_| 0);
                        buf.push(read_buf[0]);
                    }
                    match String::from_utf8(buf) {
                        Ok(value) => Primitive::String(value),
                        Err(_) => {
                            return Err("Could not decode 1-byte string (UTF-8 error)");
                        }
                    }
                }
                FormatCode::FourByteString => {
                    let mut read_buf = [0u8; 4];
                    buf_reader
                        .read_exact(&mut read_buf)
                        .await
                        .unwrap_or_else(|_| 0);
                    let len = u32::from_be_bytes(read_buf);

                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader
                            .read_exact(&mut read_buf)
                            .await
                            .unwrap_or_else(|_| 0);
                        buf.push(read_buf[0]);
                    }
                    match String::from_utf8(buf) {
                        Ok(value) => Primitive::String(value),
                        Err(_) => {
                            return Err("Could not decode 4-byte string (UTF-8 error)");
                        }
                    }
                }
                FormatCode::OneByteSymbol => {
                    let mut read_buf = [0u8; 1];
                    buf_reader
                        .read_exact(&mut read_buf)
                        .await
                        .unwrap_or_else(|_| 0);
                    let len = read_buf[0];

                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader
                            .read_exact(&mut read_buf)
                            .await
                            .unwrap_or_else(|_| 0);
                        buf.push(read_buf[0]);
                    }
                    Primitive::Symbol(buf)
                }
                FormatCode::FourByteSymbol => {
                    let mut read_buf = [0u8; 4];
                    buf_reader
                        .read_exact(&mut read_buf)
                        .await
                        .unwrap_or_else(|_| 0);
                    let len = u32::from_be_bytes(read_buf);

                    let mut buf = Vec::<u8>::with_capacity(len as usize);
                    let mut read_buf = [0u8; 1];
                    for _ in 0..len {
                        buf_reader
                            .read_exact(&mut read_buf)
                            .await
                            .unwrap_or_else(|_| 0);
                        buf.push(read_buf[0]);
                    }
                    Primitive::Symbol(buf)
                }
                FormatCode::List0 => Primitive::EmptyList,
                FormatCode::List8 => {
                    let mut read_buf = [0u8; 2];
                    buf_reader
                        .read_exact(&mut read_buf)
                        .await
                        .unwrap_or_else(|_| 0);
                    let _size = read_buf[0];
                    let count = read_buf[1];

                    let len = count as usize;
                    let mut buf = Vec::with_capacity(len);
                    for _ in 0..len {
                        let mut read_buf = [0u8; 2];
                        buf_reader
                            .read_exact(&mut read_buf)
                            .await
                            .unwrap_or_else(|_| 0);
                        let elt_fcode = FormatCode::try_from(u16::from_be_bytes(read_buf))?;
                        let elt = Box::pin(Constructor::new(elt_fcode, buf_reader)).await;
                        match elt {
                            Ok(constructor) => buf.push(Box::pin(constructor)),
                            Err(_) => return Err("Could not read element of 8-byte list"),
                        }
                    }
                    Primitive::List(buf)
                }
                FormatCode::List32 => {
                    let mut read_buf = [0u8; 8];
                    buf_reader
                        .read_exact(&mut read_buf)
                        .await
                        .unwrap_or_else(|_| 0);
                    let _size =
                        u32::from_be_bytes([read_buf[0], read_buf[1], read_buf[2], read_buf[3]]);
                    let count =
                        u32::from_be_bytes([read_buf[4], read_buf[5], read_buf[6], read_buf[7]]);

                    let len = count as usize;
                    let mut buf = Vec::with_capacity(len);
                    for _ in 0..len {
                        let mut read_buf = [0u8; 2];
                        buf_reader
                            .read_exact(&mut read_buf)
                            .await
                            .unwrap_or_else(|_| 0);
                        let elt_fcode = FormatCode::try_from(u16::from_be_bytes(read_buf))?;
                        match Box::pin(Constructor::new(elt_fcode, buf_reader)).await {
                            Ok(constructor) => buf.push(Box::pin(constructor)),
                            Err(_) => return Err("Could not read element of 32-byte list"),
                        }
                    }
                    Primitive::List(buf)
                }
                FormatCode::Map8 => {
                    let mut read_buf = [0u8; 1];
                    buf_reader
                        .read_exact(&mut read_buf)
                        .await
                        .unwrap_or_else(|_| 0);
                    let len = read_buf[0];
                    if len % 2 != 0 {
                        return Err("Map8 length found to be odd");
                    }
                    let mut buf = HashMap::with_capacity(len as usize);
                    for _ in 0..len {
                        let mut read_buf = [0u8; 2];
                        buf_reader
                            .read_exact(&mut read_buf)
                            .await
                            .unwrap_or_else(|_| 0);
                        let key_fcode = FormatCode::try_from(u16::from_be_bytes(read_buf))?;
                        let key = match Box::pin(Constructor::new(key_fcode, buf_reader)).await {
                            Ok(constructor) => constructor,
                            Err(_) => return Err("Could not read a key of an 8-byte map"),
                        };

                        buf_reader
                            .read_exact(&mut read_buf)
                            .await
                            .unwrap_or_else(|_| 0);
                        let val_fcode = FormatCode::try_from(u16::from_be_bytes(read_buf))?;
                        let val = match Box::pin(Constructor::new(val_fcode, buf_reader)).await {
                            Ok(constructor) => constructor,
                            Err(_) => return Err("Could not read a value of an 8-byte map"),
                        };
                        buf.insert(Box::pin(key), Box::pin(val));
                    }
                    Primitive::Map(InnerMap { value: buf })
                }
                FormatCode::Map32 => {
                    let mut read_buf = [0u8; 4];
                    buf_reader
                        .read_exact(&mut read_buf)
                        .await
                        .unwrap_or_else(|_| 0);
                    let len = u32::from_be_bytes(read_buf);
                    let mut buf = HashMap::with_capacity(len as usize);
                    for _ in 0..len {
                        let mut read_buf = [0u8; 2];
                        buf_reader
                            .read_exact(&mut read_buf)
                            .await
                            .unwrap_or_else(|_| 0);
                        let key_fcode = FormatCode::try_from(u16::from_be_bytes(read_buf))?;
                        let key = match Box::pin(Constructor::new(key_fcode, buf_reader)).await {
                            Ok(constructor) => constructor,
                            Err(_) => return Err("Could not read a key of an 32-byte map"),
                        };

                        buf_reader
                            .read_exact(&mut read_buf)
                            .await
                            .unwrap_or_else(|_| 0);
                        let val_fcode = FormatCode::try_from(u16::from_be_bytes(read_buf))?;
                        let val = match Box::pin(Constructor::new(val_fcode, buf_reader)).await {
                            Ok(constructor) => constructor,
                            Err(_) => return Err("Could not read a value of an 32-byte map"),
                        };
                        buf.insert(Box::pin(key), Box::pin(val));
                    }
                    Primitive::Map(InnerMap { value: buf })
                }
                FormatCode::Array8 => {
                    let mut read_buf = [0u8; 1];
                    buf_reader
                        .read_exact(&mut read_buf)
                        .await
                        .unwrap_or_else(|_| 0);
                    let _size = read_buf[0];
                    buf_reader
                        .read_exact(&mut read_buf)
                        .await
                        .unwrap_or_else(|_| 0);
                    let count = read_buf[0];

                    let read_buf = [0u8; 2];
                    let elt_constructor_code = FormatCode::try_from(u16::from_be_bytes(read_buf))?;
                    let len = count as usize;
                    let mut buf = Vec::<Primitive>::with_capacity(len);

                    for _ in 0..len {
                        // TODO: fix! this will read some primitive bytes
                        // into the descriptor of the new constructor every
                        // time!!!
                        match Box::pin(Constructor::new(elt_constructor_code, buf_reader)).await {
                            Ok(
                                Constructor::PrimitiveType(elt_primitive)
                                | Constructor::DescribedType(_, elt_primitive),
                            ) => {
                                buf.push(elt_primitive);
                            }
                            Err(_) => {
                                return Err("Could not read array8 element");
                            }
                        }
                    }
                    Primitive::Array(buf)
                }
                FormatCode::Array32 => {
                    let mut read_buf = [0u8; 4];
                    buf_reader
                        .read_exact(&mut read_buf)
                        .await
                        .unwrap_or_else(|_| 0);
                    let _size = u32::from_be_bytes(read_buf);
                    buf_reader
                        .read_exact(&mut read_buf)
                        .await
                        .unwrap_or_else(|_| 0);
                    let count = u32::from_be_bytes(read_buf);

                    let read_buf = [0u8; 2];
                    let elt_constructor_code = FormatCode::try_from(u16::from_be_bytes(read_buf))?;
                    let len = count as usize;
                    let mut buf = Vec::<Primitive>::with_capacity(len);

                    for _ in 0..len {
                        // TODO: fix! this will read some primitive bytes
                        // into the descriptor of the new constructor every
                        // time!!!
                        match Box::pin(Constructor::new(elt_constructor_code, buf_reader)).await {
                            Ok(
                                Constructor::PrimitiveType(elt_primitive)
                                | Constructor::DescribedType(_, elt_primitive),
                            ) => {
                                buf.push(elt_primitive);
                            }
                            Err(_) => {
                                return Err("Could not read array32 element");
                            }
                        }
                    }
                    Primitive::Array(buf)
                }
            })),
        }
    }
}
