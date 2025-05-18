use std::{collections::HashMap, ops::Deref, slice::Iter, time::Duration};

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
        remote_channel: Option<u16>,
        next_outgoing_id: u32,
        incoming_window: u32,
        outgoing_window: u32,
        handle_max: u32,
        offered_capabilities: Vec<Vec<u8>>,
        desired_capabilities: Vec<Vec<u8>>,
        properties: HashMap<Constructor, Constructor>,
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
            container_id: match read_string(&mut field_iter, true) {
                Ok(Some(value)) => value,
                Ok(None) | Err(_) => {
                    return Err("Mandatory field empty: container_id");
                }
            },
            // <field name="hostname" type="string"/>
            hostname: match read_string(&mut field_iter, false) {
                Ok(value) => value,
                Err(_) => {
                    return Err("Cannot read hostname");
                }
            },
            // <field name="max-frame-size" type="uint" default="4294967295"/>
            max_frame_size: match read_uint(&mut field_iter, true, Some(4294967295)) {
                Ok(Some(value)) => value,
                _ => {
                    return Err("Cannot read max_frame_size");
                }
            },
            // <field name="channel-max" type="ushort" default="65535"/>
            channel_max: match read_ushort(&mut field_iter, true, Some(65535)) {
                Ok(Some(value)) => value,
                _ => {
                    return Err("Cannot read channel_max");
                }
            },
            // <field name="idle-time-out" type="milliseconds"/>
            idle_time_out: match read_uint(&mut field_iter, false, None) {
                Ok(Some(value)) => Some(Duration::from_millis(value as u64)),
                Ok(None) => None,
                _ => {
                    return Err("Invalid field type at idle_time_out");
                }
            },
            // <type name="ietf-language-tag" class="restricted" source="symbol"/>
            // <field name="outgoing-locales" type="ietf-language-tag" multiple="true"/>
            outgoing_locales: match read_symbol_array(&mut field_iter) {
                Ok(value) => value,
                Err(_) => {
                    return Err("Cannot read outgoing_locales");
                }
            },
            // <field name="incoming-locales" type="ietf-language-tag" multiple="true"/>
            incoming_locales: match read_symbol_array(&mut field_iter) {
                Ok(value) => value,
                Err(_) => {
                    return Err("Cannot read incoming_locales");
                }
            },
            // <field name="offered-capabilities" type="symbol" multiple="true"/>
            offered_capabilities: match read_symbol_array(&mut field_iter) {
                Ok(value) => value,
                Err(_) => {
                    return Err("Cannot read offered_capabilities");
                }
            },
            // <field name="desired-capabilities" type="symbol" multiple="true"/>
            desired_capabilities: match read_symbol_array(&mut field_iter) {
                Ok(value) => value,
                Err(_) => {
                    return Err("Cannot read desired_capabilities");
                }
            },
            // <type name="fields" class="restricted" source="map"/>
            // <field name="properties" type="fields"/>
            properties: match read_map(&mut field_iter) {
                Ok(value) => value,
                Err(_) => {
                    return Err("Cannot read properties");
                }
            },
        })
    }
    // </type>

    // <type name="begin" class="composite" source="list" provides="frame">
    // <descriptor name="amqp:begin:list" code="0x00000000:0x00000011"/>
    fn begin(fields: Vec<Constructor>) -> Result<Self, &'static str> {
        let mut field_iter = fields.iter();
        Ok(Performative::Begin {
            // <field name="remote-channel" type="ushort"/>
            remote_channel: match read_ushort(&mut field_iter, false, None) {
                Ok(value) => value,
                Err(_) => {
                    return Err("Could not read field remote_channel");
                }
            },
            // <type name="transfer-number" class="restricted" source="sequence-no"/>
            // <type name="sequence-no" class="restricted" source="uint"/>
            // <field name="next-outgoing-id" type="transfer-number" mandatory="true"/>
            next_outgoing_id: match read_uint(&mut field_iter, true, None) {
                Ok(Some(value)) => value,
                _ => {
                    return Err("Could not read next_outgoing_id");
                }
            },
            // <field name="incoming-window" type="uint" mandatory="true"/>
            incoming_window: match read_uint(&mut field_iter, true, None) {
                Ok(Some(value)) => value,
                _ => {
                    return Err("Could not read incoming_window");
                }
            },
            // <field name="outgoing-window" type="uint" mandatory="true"/>
            outgoing_window: match read_uint(&mut field_iter, true, None) {
                Ok(Some(value)) => value,
                _ => {
                    return Err("Could not read outgoing_window");
                }
            },
            // <type name="handle" class="restricted" source="uint"/>
            // <field name="handle-max" type="handle" default="4294967295"/>
            handle_max: match read_uint(&mut field_iter, true, Some(4294967295)) {
                Ok(Some(value)) => value,
                _ => {
                    return Err("Cannot read handle_max");
                }
            },
            // <field name="offered-capabilities" type="symbol" multiple="true"/>
            offered_capabilities: match read_symbol_array(&mut field_iter) {
                Ok(value) => value,
                Err(_) => {
                    return Err("Cannot read offered_capabilities");
                }
            },
            // <field name="desired-capabilities" type="symbol" multiple="true"/>
            desired_capabilities: match read_symbol_array(&mut field_iter) {
                Ok(value) => value,
                Err(_) => {
                    return Err("Cannot read desired_capabilities");
                }
            },
            // <field name="properties" type="fields"/>
            properties: match read_map(&mut field_iter) {
                Ok(value) => value,
                Err(_) => {
                    return Err("Cannot read properties");
                }
            },
        })
        // </type>
    }

    // <type name="attach" class="composite" source="list" provides="frame">
    // <descriptor name="amqp:attach:list" code="0x00000000:0x00000012"/>
    // <field name="name" type="string" mandatory="true"/>
    // <field name="handle" type="handle" mandatory="true"/>
    // <type name="role" class="restricted" source="boolean">
    //     <choice name="sender" value="false"/>
    //     <choice name="receiver" value="true"/>
    // </type>
    // <field name="role" type="role" mandatory="true"/>
    // <type name="sender-settle-mode" class="restricted" source="ubyte">
    //     <choice name="unsettled" value="0"/>
    //     <choice name="settled" value="1"/>
    //     <choice name="mixed" value="2"/>
    // </type>
    // <field name="snd-settle-mode" type="sender-settle-mode" default="mixed"/>
    // <type name="receiver-settle-mode" class="restricted" source="ubyte">
    //     <choice name="first" value="0"/>
    //     <choice name="second" value="1"/>
    // </type>
    // <field name="rcv-settle-mode" type="receiver-settle-mode" default="first"/>
    // spec:wildcard[*]: A value of any type is permitted.
    // <field name="source" type="*" requires="source"/>
    // <field name="target" type="*" requires="target"/>
    // <field name="unsettled" type="map"/>
    // <field name="incomplete-unsettled" type="boolean" default="false"/>
    // <field name="initial-delivery-count" type="sequence-no"/>
    // <field name="max-message-size" type="ulong"/>
    // <field name="offered-capabilities" type="symbol" multiple="true"/>
    // <field name="desired-capabilities" type="symbol" multiple="true"/>
    // <field name="properties" type="fields"/>
    // </type>

    // <type name="flow" class="composite" source="list" provides="frame">
    // <descriptor name="amqp:flow:list" code="0x00000000:0x00000013"/>
    // <field name="next-incoming-id" type="transfer-number"/>
    // <field name="incoming-window" type="uint" mandatory="true"/>
    // <field name="next-outgoing-id" type="transfer-number" mandatory="true"/>
    // <field name="outgoing-window" type="uint" mandatory="true"/>
    // <field name="handle" type="handle"/>
    // <field name="delivery-count" type="sequence-no"/>
    // <field name="link-credit" type="uint"/>
    // <field name="available" type="uint"/>
    // <field name="drain" type="boolean" default="false"/>
    // <field name="echo" type="boolean" default="false"/>
    // <field name="properties" type="fields"/>
    // </type>

    // <type name="transfer" class="composite" source="list" provides="frame">
    // <descriptor name="amqp:transfer:list" code="0x00000000:0x00000014"/>
    // <field name="handle" type="handle" mandatory="true"/>
    // <field name="delivery-id" type="delivery-number"/>
    // <field name="delivery-tag" type="delivery-tag"/>
    // <field name="message-format" type="message-format"/>
    // <field name="settled" type="boolean"/>
    // <field name="more" type="boolean" default="false"/>
    // <field name="rcv-settle-mode" type="receiver-settle-mode"/>
    // <field name="state" type="*" requires="delivery-state"/>
    // <field name="resume" type="boolean" default="false"/>
    // <field name="aborted" type="boolean" default="false"/>
    // <field name="batchable" type="boolean" default="false"/>
    // </type>

    // <type name="disposition" class="composite" source="list" provides="frame">
    // <descriptor name="amqp:disposition:list" code="0x00000000:0x00000015"/>
    // <field name="role" type="role" mandatory="true"/>
    // <field name="first" type="delivery-number" mandatory="true"/>
    // <field name="last" type="delivery-number"/>
    // <field name="settled" type="boolean" default="false"/>
    // <field name="state" type="*" requires="delivery-state"/>
    // <field name="batchable" type="boolean" default="false"/>
    // </type>

    // <type name="detach" class="composite" source="list" provides="frame">
    // <descriptor name="amqp:detach:list" code="0x00000000:0x00000016"/>
    // <field name="handle" type="handle" mandatory="true"/>
    // <field name="closed" type="boolean" default="false"/>
    // <field name="error" type="error"/>
    // </type>

    // <type name="end" class="composite" source="list" provides="frame">
    // <descriptor name="amqp:end:list" code="0x00000000:0x00000017"/>
    // <field name="error" type="error"/>
    // </type>

    // <type name="close" class="composite" source="list" provides="frame">
    // <descriptor name="amqp:close:list" code="0x00000000:0x00000018"/>
    // <field name="error" type="error"/>
    // </type>
}

fn read_string(
    field_iter: &mut Iter<Constructor>,
    mandatory: bool,
) -> Result<Option<String>, &'static str> {
    match (*field_iter).next() {
        Some(Constructor::PrimitiveType(Primitive::String(value))) => Ok(Some(value.clone())),
        Some(Constructor::PrimitiveType(Primitive::Null)) => {
            if mandatory {
                return Err("Non-mandatory string is null");
            } else {
                Ok(None)
            }
        }
        _ => {
            return Err("Invalid type: string expected");
        }
    }
}

fn read_uint(
    field_iter: &mut Iter<Constructor>,
    mandatory: bool,
    default: Option<u32>,
) -> Result<Option<u32>, &'static str> {
    match field_iter.next() {
        Some(Constructor::PrimitiveType(Primitive::UInt(value))) => Ok(Some(*value)),
        Some(Constructor::PrimitiveType(Primitive::Null)) => {
            if !mandatory {
                Ok(None)
            } else if default.is_some() {
                Ok(default)
            } else {
                return Err("Mandatory uint field is null");
            }
        }
        _ => {
            return Err("Invalid field type at max_frame_size");
        }
    }
}

fn read_ushort(
    field_iter: &mut Iter<Constructor>,
    mandatory: bool,
    default: Option<u16>,
) -> Result<Option<u16>, &'static str> {
    match field_iter.next() {
        Some(Constructor::PrimitiveType(Primitive::UShort(value))) => Ok(Some(*value)),
        Some(Constructor::PrimitiveType(Primitive::Null)) => {
            if !mandatory {
                Ok(None)
            } else if default.is_some() {
                Ok(default)
            } else {
                return Err("Mandatory ushort field is null");
            }
        }
        _ => {
            return Err("Invalid field type at channel_max");
        }
    }
}

fn read_symbol_array(field_iter: &mut Iter<Constructor>) -> Result<Vec<Vec<u8>>, &'static str> {
    match field_iter.next() {
        Some(Constructor::PrimitiveType(Primitive::Array(value))) => {
            let mut result = vec![];
            for elt in value.iter() {
                match elt {
                    Primitive::Symbol(symbols) => {
                        result.push(symbols.clone());
                    }
                    _ => {
                        return Err("Invalid symbol array element type");
                    }
                }
            }
            Ok(result)
        }
        Some(Constructor::PrimitiveType(Primitive::EmptyList)) => Ok(vec![]),
        _ => Err("Invalid field type: symbol array expected"),
    }
}

fn read_map(
    field_iter: &mut Iter<Constructor>,
) -> Result<HashMap<Constructor, Constructor>, &'static str> {
    match field_iter.next() {
        Some(Constructor::PrimitiveType(Primitive::Map(value))) => {
            let mut result = HashMap::with_capacity(value.value.len());
            for (key, val) in value.value.iter() {
                result.insert(key.deref().clone(), val.deref().clone());
            }
            Ok(result)
        }
        _ => Err("Invalid field type: map expected"),
    }
}
