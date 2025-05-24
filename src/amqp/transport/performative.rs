use std::{collections::HashMap, ops::Deref, slice::Iter, time::Duration};

use tokio::io::AsyncReadExt;

use crate::amqp::types::{constructor::Constructor, format_code::FormatCode, primitive::Primitive};

// <type name="error" class="composite" source="list">
// <descriptor name="amqp:error:list" code="0x00000000:0x0000001d"/>
// </type>
struct PerformativeError {
    // <field name="condition" type="symbol" requires="error-condition" mandatory="true"/>
    condition: Vec<Vec<u8>>,
    // <field name="description" type="string"/>
    description: Option<String>,
    // <field name="info" type="fields"/>
    info: HashMap<Constructor, Constructor>,
}

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
        name: String,
        handle: u32,
        role: bool,
        snd_settle_mode: u8,
        rcv_settle_mode: u8,
        source: Constructor,
        target: Constructor,
        unsettled: HashMap<Constructor, Constructor>,
        incomplete_unsettled: bool,
        initial_delivery_count: u32,
        max_message_size: u64,
        offered_capabilities: Vec<Vec<u8>>,
        desired_capabilities: Vec<Vec<u8>>,
        properties: HashMap<Constructor, Constructor>,
    },
    Flow {
        next_incoming_id: Option<u32>,
        incoming_window: u32,
        next_outgoing_id: u32,
        outgoing_window: u32,
        handle: Option<u32>,
        delivery_count: Option<u32>,
        link_credit: u32,
        available: u32,
        drain: bool,
        echo: bool,
        properties: HashMap<Constructor, Constructor>,
    },
    Transfer {
        handle: u32,
        delivery_id: Option<u32>,
        delivery_tag: Vec<u8>,
        message_format: Option<u32>,
        settled: Option<bool>,
        more: bool,
        rcv_settle_mode: Option<u8>,
        state: Constructor,
        resume: bool,
        aborted: bool,
        batchable: bool,
    },
    Disposition {
        role: bool,
        first: u32,
        last: Option<u32>,
        settled: bool,
        state: Constructor,
        batchable: bool,
    },
    Detach {
        handle: u32,
        closed: bool,
        error: Option<PerformativeError>,
    },
    End {
        error: Option<PerformativeError>,
    },
    Close {
        error: Option<PerformativeError>,
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
                    let field = field.deref().clone();
                    fields.push(field);
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
            container_id: read_string(&mut field_iter, true)?
                .ok_or("Mandatory field: container_id")?,
            // <field name="hostname" type="string"/>
            hostname: read_string(&mut field_iter, false)?,
            // <field name="max-frame-size" type="uint" default="4294967295"/>
            max_frame_size: read_uint(&mut field_iter, true, Some(4294967295))?
                .ok_or("Mandatory field: max_frame_size")?,
            // <field name="channel-max" type="ushort" default="65535"/>
            channel_max: read_ushort(&mut field_iter, true, Some(65535))?
                .ok_or("Mandatory field: channel_max")?,
            // <field name="idle-time-out" type="milliseconds"/>
            idle_time_out: if let Some(ms) = read_uint(&mut field_iter, false, None)? {
                Some(Duration::from_millis(ms as u64))
            } else {
                None
            },
            // <type name="ietf-language-tag" class="restricted" source="symbol"/>
            // <field name="outgoing-locales" type="ietf-language-tag" multiple="true"/>
            outgoing_locales: read_symbol_array(&mut field_iter)?,
            // <field name="incoming-locales" type="ietf-language-tag" multiple="true"/>
            incoming_locales: read_symbol_array(&mut field_iter)?,
            // <field name="offered-capabilities" type="symbol" multiple="true"/>
            offered_capabilities: read_symbol_array(&mut field_iter)?,
            // <field name="desired-capabilities" type="symbol" multiple="true"/>
            desired_capabilities: read_symbol_array(&mut field_iter)?,
            // <type name="fields" class="restricted" source="map"/>
            // <field name="properties" type="fields"/>
            properties: read_map(&mut field_iter)?,
        })
    }
    // </type>

    // <type name="begin" class="composite" source="list" provides="frame">
    // <descriptor name="amqp:begin:list" code="0x00000000:0x00000011"/>
    fn begin(fields: Vec<Constructor>) -> Result<Self, &'static str> {
        let mut field_iter = fields.iter();
        Ok(Performative::Begin {
            // <field name="remote-channel" type="ushort"/>
            remote_channel: read_ushort(&mut field_iter, false, None)?,
            // <type name="sequence-no" class="restricted" source="uint"/>
            // <type name="transfer-number" class="restricted" source="sequence-no"/>
            // <field name="next-outgoing-id" type="transfer-number" mandatory="true"/>
            next_outgoing_id: read_uint(&mut field_iter, true, None)?
                .ok_or("Mandatory field: next_outgoing_id")?,
            // <field name="incoming-window" type="uint" mandatory="true"/>
            incoming_window: read_uint(&mut field_iter, true, None)?
                .ok_or("Mandatory field: incoming_window")?,
            // <field name="outgoing-window" type="uint" mandatory="true"/>
            outgoing_window: read_uint(&mut field_iter, true, None)?
                .ok_or("Mandatory field: outgoing_window")?,
            // <type name="handle" class="restricted" source="uint"/>
            // <field name="handle-max" type="handle" default="4294967295"/>
            handle_max: read_uint(&mut field_iter, true, Some(4294967295))?
                .ok_or("Mandatory field: handle_max")?,
            // <field name="offered-capabilities" type="symbol" multiple="true"/>
            offered_capabilities: read_symbol_array(&mut field_iter)?,
            // <field name="desired-capabilities" type="symbol" multiple="true"/>
            desired_capabilities: read_symbol_array(&mut field_iter)?,
            // <field name="properties" type="fields"/>
            properties: read_map(&mut field_iter)?,
        })
        // </type>
    }

    // <type name="attach" class="composite" source="list" provides="frame">
    // <descriptor name="amqp:attach:list" code="0x00000000:0x00000012"/>
    fn attach(fields: Vec<Constructor>) -> Result<Self, &'static str> {
        let mut field_iter = fields.iter();
        Ok(Performative::Attach {
            // <field name="name" type="string" mandatory="true"/>
            name: read_string(&mut field_iter, true)?.ok_or("Mandatory field: name")?,
            // <field name="handle" type="handle" mandatory="true"/>
            handle: read_uint(&mut field_iter, true, None)?.ok_or("Mandatory field: handle")?,
            // <type name="role" class="restricted" source="boolean">
            //     <choice name="sender" value="false"/>
            //     <choice name="receiver" value="true"/>
            // </type>
            // <field name="role" type="role" mandatory="true"/>
            role: read_bool(&mut field_iter, true, None)?.ok_or("Mandatory field: role")?,
            // <type name="sender-settle-mode" class="restricted" source="ubyte">
            //     <choice name="unsettled" value="0"/>
            //     <choice name="settled" value="1"/>
            //     <choice name="mixed" value="2"/>
            // </type>
            // <field name="snd-settle-mode" type="sender-settle-mode" default="mixed"/>
            snd_settle_mode: read_ubyte(&mut field_iter, true, Some(2))?
                .ok_or("Mandatory field: snd_settle_mode")?,
            // <type name="receiver-settle-mode" class="restricted" source="ubyte">
            //     <choice name="first" value="0"/>
            //     <choice name="second" value="1"/>
            // </type>
            // <field name="rcv-settle-mode" type="receiver-settle-mode" default="first"/>
            rcv_settle_mode: read_ubyte(&mut field_iter, true, Some(0))?
                .ok_or("Mandatory field: rcv_settle_mode")?,
            // spec:wildcard[*]: A value of any type is permitted.
            // <field name="source" type="*" requires="source"/>
            source: field_iter.next().ok_or("Mandatory field: source")?.clone(),
            // <field name="target" type="*" requires="target"/>
            target: field_iter.next().ok_or("Mandatory field: target")?.clone(),
            // <field name="unsettled" type="map"/>
            unsettled: read_map(&mut field_iter)?,
            // <field name="incomplete-unsettled" type="boolean" default="false"/>
            incomplete_unsettled: read_bool(&mut field_iter, true, Some(false))?
                .ok_or("Mandatory field: incomplete_unsettled")?,
            // <field name="initial-delivery-count" type="sequence-no"/>
            initial_delivery_count: read_uint(&mut field_iter, false, None)?
                .ok_or("Mandatory field: initial_delivery_count")?,
            // <field name="max-message-size" type="ulong"/>
            max_message_size: read_ulong(&mut field_iter, false, None)?
                .ok_or("Mandatory field: max_message_size")?,
            // <field name="offered-capabilities" type="symbol" multiple="true"/>
            offered_capabilities: read_symbol_array(&mut field_iter)?,
            // <field name="desired-capabilities" type="symbol" multiple="true"/>
            desired_capabilities: read_symbol_array(&mut field_iter)?,
            // <field name="properties" type="fields"/>
            properties: read_map(&mut field_iter)?,
        })
    }
    // </type>

    // <type name="flow" class="composite" source="list" provides="frame">
    // <descriptor name="amqp:flow:list" code="0x00000000:0x00000013"/>
    fn flow(fields: Vec<Constructor>) -> Result<Self, &'static str> {
        let mut field_iter = fields.iter();
        Ok(Self::Flow {
            // <field name="next-incoming-id" type="transfer-number"/>
            next_incoming_id: read_uint(&mut field_iter, false, None)?,
            // <field name="incoming-window" type="uint" mandatory="true"/>
            incoming_window: read_uint(&mut field_iter, true, None)?
                .ok_or("Mandatory field: incoming_window")?,
            // <field name="next-outgoing-id" type="transfer-number" mandatory="true"/>
            next_outgoing_id: read_uint(&mut field_iter, true, None)?
                .ok_or("Mandatory field: next_outgoing_id")?,
            // <field name="outgoing-window" type="uint" mandatory="true"/>
            outgoing_window: read_uint(&mut field_iter, true, None)?
                .ok_or("Mandatory field: outgoing_window")?,
            // <field name="handle" type="handle"/>
            handle: read_uint(&mut field_iter, false, None)?,
            // <field name="delivery-count" type="sequence-no"/>
            delivery_count: read_uint(&mut field_iter, true, None)?,
            // <field name="link-credit" type="uint"/>
            link_credit: read_uint(&mut field_iter, true, None)?
                .ok_or("Mandatory field: link_credit")?,
            // <field name="available" type="uint"/>
            available: read_uint(&mut field_iter, true, None)?
                .ok_or("Mandatory field: available")?,
            // <field name="drain" type="boolean" default="false"/>
            drain: read_bool(&mut field_iter, true, None)?.ok_or("Mandatory field: drain")?,
            // <field name="echo" type="boolean" default="false"/>
            echo: read_bool(&mut field_iter, true, None)?.ok_or("Mandatory field: echo")?,
            // <field name="properties" type="fields"/>
            properties: read_map(&mut field_iter)?,
        })
    }
    // </type>

    // <type name="transfer" class="composite" source="list" provides="frame">
    // <descriptor name="amqp:transfer:list" code="0x00000000:0x00000014"/>
    fn transfer(fields: Vec<Constructor>) -> Result<Self, &'static str> {
        let mut field_iter = fields.iter();
        Ok(Self::Transfer {
            // <field name="handle" type="handle" mandatory="true"/>
            handle: read_uint(&mut field_iter, true, None)?.ok_or("Mandatory field: handle")?,
            // <type name="delivery-number" class="restricted" source="sequence-no"/>
            // <field name="delivery-id" type="delivery-number"/>
            delivery_id: read_uint(&mut field_iter, false, None)?,
            // <type name="delivery-tag" class="restricted" source="binary"/>
            // A delivery-tag may be up to 32 octets of binary data.
            // <field name="delivery-tag" type="delivery-tag"/>
            delivery_tag: {
                let tag = read_binary(&mut field_iter)?;
                if tag.len() > 32 {
                    return Err("Field delivery_tag is longer than 32 octets");
                }
                tag.clone()
            },
            // <type name="message-format" class="restricted" source="uint"/>
            // <field name="message-format" type="message-format"/>
            message_format: read_uint(&mut field_iter, false, None)?,
            // <field name="settled" type="boolean"/>
            settled: read_bool(&mut field_iter, false, None)?,
            // <field name="more" type="boolean" default="false"/>
            more: read_bool(&mut field_iter, true, Some(false))?
                .ok_or("the field more is null unexpectedly")?,
            // <field name="rcv-settle-mode" type="receiver-settle-mode"/>
            rcv_settle_mode: read_ubyte(&mut field_iter, false, None)?,
            // <field name="state" type="*" requires="delivery-state"/>
            state: field_iter.next().ok_or("Mandatory field: state")?.clone(),
            // <field name="resume" type="boolean" default="false"/>
            resume: read_bool(&mut field_iter, true, Some(false))?
                .ok_or("the field resume is null unexpectedly")?,
            // <field name="aborted" type="boolean" default="false"/>
            aborted: read_bool(&mut field_iter, true, Some(false))?
                .ok_or("the field aborted is null unexpectedly")?,
            // <field name="batchable" type="boolean" default="false"/>
            batchable: read_bool(&mut field_iter, true, Some(false))?
                .ok_or("the field batchable is null unexpectedly")?,
        })
    }
    // </type>

    // <type name="disposition" class="composite" source="list" provides="frame">
    // <descriptor name="amqp:disposition:list" code="0x00000000:0x00000015"/>
    fn disposition(fields: Vec<Constructor>) -> Result<Self, &'static str> {
        let mut field_iter = fields.iter();
        Ok(Self::Disposition {
            // <field name="role" type="role" mandatory="true"/>
            role: read_bool(&mut field_iter, true, None)?.ok_or("Mandatory field: role")?,
            // <field name="first" type="delivery-number" mandatory="true"/>
            first: read_uint(&mut field_iter, false, None)?.ok_or("Mandatory field: first")?,
            // <field name="last" type="delivery-number"/>
            last: read_uint(&mut field_iter, false, None)?,
            // <field name="settled" type="boolean" default="false"/>
            settled: read_bool(&mut field_iter, true, Some(false))?
                .ok_or("the field settled is null unexpectedly")?,
            // <field name="state" type="*" requires="delivery-state"/>
            state: field_iter.next().ok_or("Mandatory field: state")?.clone(),
            // <field name="batchable" type="boolean" default="false"/>
            batchable: read_bool(&mut field_iter, true, Some(false))?
                .ok_or("the field batchable is null unexpectedly")?,
        })
    }
    // </type>

    // <type name="detach" class="composite" source="list" provides="frame">
    // <descriptor name="amqp:detach:list" code="0x00000000:0x00000016"/>
    fn detach(fields: Vec<Constructor>) -> Result<Self, &'static str> {
        let mut field_iter = fields.iter();
        Ok(Self::Detach {
            // <field name="handle" type="handle" mandatory="true"/>
            handle: read_uint(&mut field_iter, true, None)?.ok_or("Mandatory field: handle")?,
            // <field name="closed" type="boolean" default="false"/>
            closed: read_bool(&mut field_iter, true, Some(false))?
                .ok_or("the field closed is null unexpectedly")?,
            // <field name="error" type="error"/>
            error: read_error(&mut field_iter)?,
        })
    }
    // </type>

    // <type name="end" class="composite" source="list" provides="frame">
    // <descriptor name="amqp:end:list" code="0x00000000:0x00000017"/>
    fn end(fields: Vec<Constructor>) -> Result<Self, &'static str> {
        let mut field_iter = fields.iter();
        Ok(Self::End {
            // <field name="error" type="error"/>
            error: read_error(&mut field_iter)?,
        })
    }
    // </type>

    // <type name="close" class="composite" source="list" provides="frame">
    // <descriptor name="amqp:close:list" code="0x00000000:0x00000018"/>
    fn close(fields: Vec<Constructor>) -> Result<Self, &'static str> {
        let mut field_iter = fields.iter();
        Ok(Self::Close {
            // <field name="error" type="error"/>
            error: read_error(&mut field_iter)?,
        })
    }
    // </type>
}

fn read_bool(
    field_iter: &mut Iter<Constructor>,
    mandatory: bool,
    default: Option<bool>,
) -> Result<Option<bool>, &'static str> {
    match field_iter.next() {
        Some(Constructor::PrimitiveType(Primitive::Boolean(value))) => Ok(Some(*value)),
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
            return Err("Invalid field type, expected boolean");
        }
    }
}

fn read_ubyte(
    field_iter: &mut Iter<Constructor>,
    mandatory: bool,
    default: Option<u8>,
) -> Result<Option<u8>, &'static str> {
    match field_iter.next() {
        Some(Constructor::PrimitiveType(Primitive::UByte(value))) => Ok(Some(*value)),
        Some(Constructor::PrimitiveType(Primitive::Null)) => {
            if !mandatory {
                Ok(None)
            } else if default.is_some() {
                Ok(default)
            } else {
                return Err("Mandatory ubyte field is null");
            }
        }
        _ => {
            return Err("Invalid field type, expected ubyte");
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
            return Err("Invalid field type, expected ushort");
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
            return Err("Invalid field type, expected uint");
        }
    }
}

fn read_ulong(
    field_iter: &mut Iter<Constructor>,
    mandatory: bool,
    default: Option<u64>,
) -> Result<Option<u64>, &'static str> {
    match field_iter.next() {
        Some(Constructor::PrimitiveType(Primitive::ULong(value))) => Ok(Some(*value)),
        Some(Constructor::PrimitiveType(Primitive::Null)) => {
            if !mandatory {
                Ok(None)
            } else if default.is_some() {
                Ok(default)
            } else {
                return Err("Mandatory ulong field is null");
            }
        }
        _ => {
            return Err("Invalid field type, expected ulong");
        }
    }
}

fn read_string(
    field_iter: &mut Iter<Constructor>,
    mandatory: bool,
) -> Result<Option<String>, &'static str> {
    match (*field_iter).next() {
        Some(Constructor::PrimitiveType(Primitive::String(value))) => Ok(Some(value.clone())),
        Some(Constructor::PrimitiveType(Primitive::Null)) => {
            if mandatory {
                return Err("Mandatory string is null");
            } else {
                Ok(None)
            }
        }
        _ => {
            return Err("Invalid type: string expected");
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

fn read_binary(field_iter: &mut Iter<Constructor>) -> Result<Vec<u8>, &'static str> {
    match field_iter.next() {
        Some(Constructor::PrimitiveType(Primitive::Binary(value))) => Ok(value.clone()),
        _ => Err("Invalid field type: binary expected"),
    }
}

fn read_error(
    field_iter: &mut Iter<Constructor>,
) -> Result<Option<PerformativeError>, &'static str> {
    match field_iter.next() {
        Some(constructor) => match constructor {
            Constructor::DescribedType(descriptor, list_primitive) => {
                match descriptor.deref().clone() {
                    Constructor::PrimitiveType(constructor_primitive) => {
                        match constructor_primitive {
                            Primitive::String(prim_body) => match prim_body.as_str() {
                                "amqp:error:list" => {
                                    let fields = match list_primitive {
                                        Primitive::List(boxed_fields) => {
                                            let mut fields = Vec::with_capacity(boxed_fields.len());
                                            for field in boxed_fields.iter() {
                                                let field = field.deref().clone();
                                                fields.push(field);
                                            }
                                            fields
                                        }
                                        _ => {
                                            return Err("Performative descriptor is not a list");
                                        }
                                    };
                                    let mut field_iter = fields.iter();

                                    Ok(Some(PerformativeError {
                                        condition: read_symbol_array(&mut field_iter)?,
                                        description: read_string(&mut field_iter, true)?,
                                        info: read_map(&mut field_iter)?,
                                    }))
                                }
                                _ => return Err("Unknown error type"),
                            },
                            _ => {
                                return Err("Performative constructor descriptor is not a string");
                            }
                        }
                    }
                    Constructor::DescribedType(_, _) => {
                        return Err("Performative constructor descriptor is not a primitive type");
                    }
                }
            }
            Constructor::PrimitiveType(Primitive::Null) => Ok(None),
            _ => {
                return Err("The error field is of an unexpected type");
            }
        },
        None => Ok(None),
    }
}
