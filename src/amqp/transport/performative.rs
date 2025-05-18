use std::{collections::HashMap, ops::Deref, time::Duration};

use tokio::io::AsyncReadExt;

use crate::amqp::types::{constructor::Constructor, format_code::FormatCode, primitive::Primitive};

pub enum Performative {
    Open {
        container_id: String,
        hostname: Option<String>,
        max_frame_size: u32,
        channel_max: u16,
        idle_time_out: Option<Duration>,
        outgoing_locales: Vec<Vec<u8>>,
        incoming_locales: Vec<Vec<u8>>,
        offered_capabilities: Vec<Vec<u8>>,
        desired_capabilities: Vec<Vec<u8>>,
        properties: HashMap<Constructor, Constructor>,
    },
    Begin {
        field: u16,
    },
    Attach {
        field: u16,
    },
    Flow {
        field: u16,
    },
    Transfer {
        field: u16,
    },
    Disposition {
        field: u16,
    },
    Detach {
        field: u16,
    },
    End {
        field: u16,
    },
    Close {
        field: u16,
    },
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
        let fields = match primitive {
            Primitive::List(boxed_fields) => {
                let mut fields = Vec::with_capacity(boxed_fields.len());
                for field in boxed_fields.iter() {
                    fields.push(*field.clone());
                }
                fields
            }
            _ => {
                return Err("Performative descriptor is not a list");
            }
        };
        match type_name.as_str() {
            "amqp:open:list" => Self::open(fields),
            "amqp:begin:list" => Self::begin(fields),
            "amqp:attach:list" => Self::attach(fields),
            "amqp:flow:list" => Self::flow(fields),
            "amqp:transfer:list" => Self::transfer(fields),
            "amqp:disposition:list" => Self::disposition(fields),
            "amqp:detach:list" => Self::detach(fields),
            "amqp:end:list" => Self::end(fields),
            "amqp:close:list" => Self::close(fields),
            _ => Err("Unknown performative type name"),
        }
    }

    // <type name="open" class="composite" source="list" provides="frame">
    // <descriptor name="amqp:open:list" code="0x00000000:0x00000010"/>
    fn open(fields: Vec<Constructor>) -> Result<Self, &'static str> {
        let mut field_iter = fields.iter();
        Ok(Performative::Open {
            // <field name="container-id" type="string" mandatory="true"/>
            container_id: match field_iter.next() {
                Some(Constructor::PrimitiveType(Primitive::String(value))) => value.clone(),
                _ => {
                    return Err("Invalid field type at container_id");
                }
            },
            // <field name="hostname" type="string"/>
            hostname: match field_iter.next() {
                Some(Constructor::PrimitiveType(Primitive::String(value))) => Some(value.clone()),
                Some(Constructor::PrimitiveType(Primitive::Null)) => None,
                _ => {
                    return Err("Invalid field type at hostname");
                }
            },
            // <field name="max-frame-size" type="uint" default="4294967295"/>
            max_frame_size: match field_iter.next() {
                Some(Constructor::PrimitiveType(Primitive::UInt(value))) => *value,
                Some(Constructor::PrimitiveType(Primitive::Null)) => 4294967295,
                _ => {
                    return Err("Invalid field type at max_frame_size");
                }
            },
            // <field name="channel-max" type="ushort" default="65535"/>
            channel_max: match field_iter.next() {
                Some(Constructor::PrimitiveType(Primitive::UShort(value))) => *value,
                Some(Constructor::PrimitiveType(Primitive::Null)) => 65535,
                _ => {
                    return Err("Invalid field type at channel_max");
                }
            },
            // <field name="idle-time-out" type="milliseconds"/>
            idle_time_out: match field_iter.next() {
                Some(Constructor::PrimitiveType(Primitive::UInt(value))) => {
                    Some(Duration::from_millis(*value as u64))
                }
                Some(Constructor::PrimitiveType(Primitive::Null)) => None,
                _ => {
                    return Err("Invalid field type at idle_time_out");
                }
            },
            // <type name="ietf-language-tag" class="restricted" source="symbol"/>
            // <field name="outgoing-locales" type="ietf-language-tag" multiple="true"/>
            outgoing_locales: match field_iter.next() {
                Some(Constructor::PrimitiveType(Primitive::Array(value))) => {
                    let mut result = vec![];
                    for elt in value.iter() {
                        match elt {
                            Primitive::Symbol(symbols) => {
                                result.push(symbols.clone());
                            }
                            _ => {
                                return Err("Invalid locale type at outgoing_locales");
                            }
                        }
                    }
                    result
                }
                Some(Constructor::PrimitiveType(Primitive::EmptyList)) => vec![],
                _ => {
                    return Err("Invalid field type at outgoing_locales");
                }
            },
            // <field name="incoming-locales" type="ietf-language-tag" multiple="true"/>
            incoming_locales: match field_iter.next() {
                Some(Constructor::PrimitiveType(Primitive::Array(value))) => {
                    let mut result = vec![];
                    for elt in value.iter() {
                        match elt {
                            Primitive::Symbol(symbols) => {
                                result.push(symbols.clone());
                            }
                            _ => {
                                return Err("Invalid locale type at incoming_locales");
                            }
                        }
                    }
                    result
                }
                Some(Constructor::PrimitiveType(Primitive::EmptyList)) => vec![],
                _ => {
                    return Err("Invalid field type at incoming_locales");
                }
            },
            // <field name="offered-capabilities" type="symbol" multiple="true"/>
            offered_capabilities: match field_iter.next() {
                Some(Constructor::PrimitiveType(Primitive::Array(value))) => {
                    let mut result = vec![];
                    for elt in value.iter() {
                        match elt {
                            Primitive::Symbol(symbols) => {
                                result.push(symbols.clone());
                            }
                            _ => {
                                return Err("Invalid locale type at offered_capabilities");
                            }
                        }
                    }
                    result
                }
                Some(Constructor::PrimitiveType(Primitive::Null)) => vec![],
                _ => {
                    return Err("Invalid field type at container_id");
                }
            },
            // <field name="desired-capabilities" type="symbol" multiple="true"/>
            desired_capabilities: match field_iter.next() {
                Some(Constructor::PrimitiveType(Primitive::Array(value))) => {
                    let mut result = vec![];
                    for elt in value.iter() {
                        match elt {
                            Primitive::Symbol(symbols) => {
                                result.push(symbols.clone());
                            }
                            _ => {
                                return Err("Invalid locale type at desired_capabilities");
                            }
                        }
                    }
                    result
                }
                Some(Constructor::PrimitiveType(Primitive::Null)) => vec![],
                _ => {
                    return Err("Invalid field type at container_id");
                }
            },
            // <field name="properties" type="fields"/>
            properties: match field_iter.next() {
                Some(Constructor::PrimitiveType(Primitive::Map(value))) => {
                    let mut result = HashMap::with_capacity(value.value.len());
                    for (key, val) in value.value.iter() {
                        result.insert(key.deref().clone(), val.deref().clone());
                    }
                    result
                }
                _ => {
                    return Err("Invalid field type at container_id");
                }
            },
        })
    }
    // </type>
}
