pub enum FormatCode {
    NonPrimitive = 0x00,
    Null = 0x40,           // fixed/0 the null value
    Boolean = 0x56, // fixed/1 boolean with the octet 0x00 being false and octet 0x01 being true
    BooleanTrue = 0x41, // true, fixed/0 the boolean value true
    BooleanFalse = 0x42, // false, fixed/0 the boolean value false
    Ubyte = 0x50,   // fixed/1 8-bit unsigned integer
    Ushort = 0x60,  // fixed/2 16-bit unsigned integer in network byte order
    Uint = 0x70,    // fixed/4 32-bit unsigned integer in network byte order
    Smalluint = 0x52, // fixed/1 unsigned integer value in the range 0 to 255 inclusive
    Uint0 = 0x43,   // fixed/0 the uint value 0
    Ulong = 0x80,   // fixed/8 64-bit unsigned integer in network byte order
    Smallulong = 0x53, // fixed/1 unsigned long value in the range 0 to 255 inclusive
    Ulong0 = 0x44,  // fixed/0 the ulong value 0
    Byte = 0x51,    // fixed/1 8-bit two’s-complement integer
    Short = 0x61,   // fixed/2 16-bit two’s-complement integer in network byte order
    Int = 0x71,     // fixed/4 32-bit two’s-complement integer in network byte order
    Smallint = 0x54, // fixed/1 signed integer value in the range -128 to 127 inclusive
    Long = 0x81,    // fixed/8 64-bit two’s-complement integer in network byte order
    Smalllong = 0x55, // fixed/1 signed long value in the range -128 to 127 inclusive
    Float = 0x72,   // fixed/4 IEEE 754-2008 binary32
    Double = 0x82,  // fixed/8 IEEE 754-2008 binary64
    Decimal32 = 0x74, // fixed/4 IEEE 754-2008 decimal32 using the Binary Integer Decimal encoding
    Decimal64 = 0x84, // fixed/8 IEEE 754-2008 decimal64 using the Binary Integer Decimal encoding
    Decimal128 = 0x94, // fixed/16 IEEE 754-2008 decimal128 using the Binary Integer Decimal encoding
    Char = 0x73,       // fixed/4 a UTF-32BE encoded unicode character
    Timestamp = 0x83, // fixed/8 64-bit signed integer representing milliseconds since the unix epoch
    Uuid = 0x98,      // fixed/16 UUID as defined in section 4.1.2 of RFC-4122
    OneByteBinary = 0xa0, // variable/1 up to 28 - 1 octets of binary data
    FourByteBinary = 0xb0, // variable/4 up to 232 - 1 octets of binary data
    OneByteString = 0xa1, // variable/1 up to 28 - 1 octets worth of UTF-8 unicode (with no byte order mark)
    FourByteString = 0xb1, // variable/4 up to 232 - 1 octets worth of UTF-8 unicode (with no byte order mark)
    OneByteSymbol = 0xa3, // variable/1 up to 28 - 1 seven bit ASCII characters representing a symbolic value
    FourByteSymbol = 0xb3, // variable/4 up to 232 - 1 seven bit ASCII characters representing a symbolic value
    List0 = 0x45,          // fixed/0 the empty list (i.e. the list with no elements)
    List8 = 0xc0, // compound/1 up to 28 - 1 list elements with total size less than 28 octets
    List32 = 0xd0, // compound/4 up to 232 - 1 list elements with total size less than 232 octets
    Map8 = 0xc1,  // compound/1 up to 28 - 1 octets of encoded map data
    Map32 = 0xd1, // compound/4 up to 232 - 1 octets of encoded map data
    Array8 = 0xe0, // array/1 up to 28 - 1 array elements with total size less than 28 octets
    Array32 = 0xf0, // array/4 up to 232 - 1 array elements with total size less than 232 octets
}

impl TryFrom<u16> for FormatCode {
    type Error = &'static str;

    fn try_from(fcode: u16) -> Result<Self, Self::Error> {
        match fcode {
            0x00 => Ok(FormatCode::NonPrimitive),
            0x40 => Ok(FormatCode::Null),
            0x56 => Ok(FormatCode::Boolean),
            0x41 => Ok(FormatCode::BooleanTrue),
            0x42 => Ok(FormatCode::BooleanFalse),
            0x50 => Ok(FormatCode::Ubyte),
            0x60 => Ok(FormatCode::Ushort),
            0x70 => Ok(FormatCode::Uint),
            0x52 => Ok(FormatCode::Smalluint),
            0x43 => Ok(FormatCode::Uint0),
            0x80 => Ok(FormatCode::Ulong),
            0x53 => Ok(FormatCode::Smallulong),
            0x44 => Ok(FormatCode::Ulong0),
            0x51 => Ok(FormatCode::Byte),
            0x61 => Ok(FormatCode::Short),
            0x71 => Ok(FormatCode::Int),
            0x54 => Ok(FormatCode::Smallint),
            0x81 => Ok(FormatCode::Long),
            0x55 => Ok(FormatCode::Smalllong),
            0x72 => Ok(FormatCode::Float),
            0x82 => Ok(FormatCode::Double),
            0x74 => Ok(FormatCode::Decimal32),
            0x84 => Ok(FormatCode::Decimal64),
            0x94 => Ok(FormatCode::Decimal128),
            0x73 => Ok(FormatCode::Char),
            0x83 => Ok(FormatCode::Timestamp),
            0x98 => Ok(FormatCode::Uuid),
            0xa0 => Ok(FormatCode::OneByteBinary),
            0xb0 => Ok(FormatCode::FourByteBinary),
            0xa1 => Ok(FormatCode::OneByteString),
            0xb1 => Ok(FormatCode::FourByteString),
            0xa3 => Ok(FormatCode::OneByteSymbol),
            0xb3 => Ok(FormatCode::FourByteSymbol),
            0x45 => Ok(FormatCode::List0),
            0xc0 => Ok(FormatCode::List8),
            0xd0 => Ok(FormatCode::List32),
            0xc1 => Ok(FormatCode::Map8),
            0xd1 => Ok(FormatCode::Map32),
            0xe0 => Ok(FormatCode::Array8),
            0xf0 => Ok(FormatCode::Array32),
            _ => Err("Invalid format code"),
        }
    }
}
