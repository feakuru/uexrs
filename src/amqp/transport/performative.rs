use std::ops::Deref;

use tokio::io::AsyncReadExt;

use crate::amqp::types::{constructor::Constructor, format_code::FormatCode, primitive::Primitive};

pub enum Performative {
    Open(Primitive),
    Begin(Primitive),
    Attach(Primitive),
    Flow(Primitive),
    Transfer(Primitive),
    Disposition(Primitive),
    Detach(Primitive),
    End(Primitive),
    Close(Primitive),
}

impl Performative {
    pub async fn new(buf_reader: &mut (impl AsyncReadExt + Unpin)) -> Result<Self, &'static str> {
        let mut read_buf = [0u8; 2];
        buf_reader
            .read_exact(&mut read_buf)
            .await
            .unwrap_or_else(|_| 0);
        let fcode = FormatCode::try_from(u16::from_be_bytes(read_buf))?;
        let constructor = Constructor::new(fcode, buf_reader).await.unwrap();
        match constructor {
            Constructor::PrimitiveType(_) => {
                Err("Constructor for a performative is a primitive type")
            }
            Constructor::DescribedType(descriptor, constructor_primitive) => {
                let descriptor: Constructor = descriptor.deref().clone();
                match descriptor {
                    Constructor::PrimitiveType(primitive) => match primitive {
                        Primitive::String(prim_body) => {
                            Self::decode_descriptor(prim_body, constructor_primitive).await
                        }
                        _ => Err("Performative constructor descriptor is not a string"),
                    },
                    Constructor::DescribedType(_, _) => {
                        Err("Performative constructor descriptor is not a primitive type")
                    }
                }
            }
        }
    }

    async fn decode_descriptor(
        type_name: String,
        primitive: Primitive,
    ) -> Result<Self, &'static str> {
        match type_name.as_str() {
            "amqp:open:list" => Ok(Self::Open(primitive)),
            "amqp:begin:list" => Ok(Self::Begin(primitive)),
            "amqp:attach:list" => Ok(Self::Attach(primitive)),
            "amqp:flow:list" => Ok(Self::Flow(primitive)),
            "amqp:transfer:list" => Ok(Self::Transfer(primitive)),
            "amqp:disposition:list" => Ok(Self::Disposition(primitive)),
            "amqp:detach:list" => Ok(Self::Detach(primitive)),
            "amqp:end:list" => Ok(Self::End(primitive)),
            "amqp:close:list" => Ok(Self::Close(primitive)),
            _ => Err("Unknown performative type name"),
        }
    }
}
