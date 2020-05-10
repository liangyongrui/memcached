//! [BinaryProtocol](https://github.com/memcached/memcached/wiki/BinaryProtocolRevamped)

pub(crate) const OK_STATUS: u16 = 0x0;

pub(crate) enum Opcode {
    Get = 0x00,
    Set = 0x01,
    Add = 0x02,
    Replace = 0x03,
    Delete = 0x04,
    Increment = 0x05,
    Decrement = 0x06,
    Flush = 0x08,
    Stat = 0x10,
    Noop = 0x0a,
    Version = 0x0b,
    GetKQ = 0x0d,
    Append = 0x0e,
    Prepend = 0x0f,
    Touch = 0x1c,
    StartAuth = 0x21,
}

pub(crate) enum Magic {
    Request = 0x80,
    Response = 0x81,
}
