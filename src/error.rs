use std::borrow::Cow;
use std::error;
use std::fmt;
use std::io;
use std::num;
use std::str;
use std::string;

/// Client-side errors
#[derive(Debug, PartialEq)]
pub enum ClientError {
    /// The key provided was longer than 250 bytes.
    KeyTooLong,
    /// The server returned an error prefixed with CLIENT_ERROR in response to a command.
    Error(Cow<'static, str>),
    ///connections is empty
    ConnectionsIsEmpty,
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClientError::KeyTooLong => write!(f, "The provided key was too long."),
            ClientError::ConnectionsIsEmpty => write!(f, "The Connections is empty."),
            ClientError::Error(s) => write!(f, "{s}"),
        }
    }
}

impl From<ClientError> for MemcachedError {
    fn from(err: ClientError) -> Self {
        MemcachedError::ClientError(err)
    }
}

/// Server-side errors
#[derive(Debug)]
pub enum ServerError {
    /// When using binary protocol, the server returned magic byte other
    /// than 0x81 in the response packet.
    BadMagic(u8),
    /// The client did not expect this response from the server.
    BadResponse(Cow<'static, str>),
    /// The server returned an error prefixed with SERVER_ERROR in response to a command.
    Error(String),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ServerError::BadMagic(e) => write!(
                f,
                "Expected 0x81 as magic in response header, but found: {e:x}"
            ),
            ServerError::BadResponse(s) => write!(f, "Unexpected: {s} in response"),
            ServerError::Error(s) => write!(f, "{s}"),
        }
    }
}

/// Command specific errors.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CommandError {
    /// The client tried to set a key which already existed in the server.
    KeyExists,
    /// The client tried to set a key which does not exist in the server.
    KeyNotFound,
    /// The value for a key was too large. The limit is usually 1MB.
    ValueTooLarge,
    /// Invalid arguments were passed to the command.
    InvalidArguments,
    /// The server requires authentication.
    AuthenticationRequired,
    ///Incr/Decr on non-numeric value.
    IncrOrDecrOnNonNumericValue,
    /// When using binary protocol, the server returned an unknown response status.
    Unknown(u16),
    /// The client sent an invalid command to the server.
    InvalidCommand,
}

impl From<String> for ClientError {
    fn from(s: String) -> Self {
        ClientError::Error(Cow::Owned(s))
    }
}

impl From<String> for ServerError {
    fn from(s: String) -> Self {
        ServerError::Error(s)
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandError::KeyExists => write!(f, "Key already exists in the server."),
            CommandError::KeyNotFound => write!(f, "Key was not found in the server."),
            CommandError::ValueTooLarge => write!(f, "Value was too large."),
            CommandError::InvalidArguments => write!(f, "Invalid arguments provided."),
            CommandError::AuthenticationRequired => write!(f, "Authentication required."),
            CommandError::IncrOrDecrOnNonNumericValue => {
                write!(f, "Incr/Decr on non-numeric value.")
            }
            CommandError::Unknown(code) => write!(f, "Unknown error occurred with code: {code}."),
            CommandError::InvalidCommand => write!(f, "Invalid command sent to the server."),
        }
    }
}

impl From<u16> for CommandError {
    fn from(status: u16) -> CommandError {
        match status {
            0x1 => CommandError::KeyNotFound,
            0x2 => CommandError::KeyExists,
            0x3 => CommandError::ValueTooLarge,
            0x4 => CommandError::InvalidArguments,
            0x6 => CommandError::IncrOrDecrOnNonNumericValue,
            0x20 => CommandError::AuthenticationRequired,
            e => CommandError::Unknown(e),
        }
    }
}

impl From<CommandError> for MemcachedError {
    fn from(err: CommandError) -> Self {
        MemcachedError::CommandError(err)
    }
}

impl From<ServerError> for MemcachedError {
    fn from(err: ServerError) -> Self {
        MemcachedError::ServerError(err)
    }
}
#[allow(missing_docs)]
#[derive(Debug)]
pub enum ParseError {
    Bool(str::ParseBoolError),
    Int(num::ParseIntError),
    Float(num::ParseFloatError),
    String(string::FromUtf8Error),
    Str(str::Utf8Error),
    Url(url::ParseError),
    Bincode(bincode::Error),
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ParseError::Bool(ref e) => e.source(),
            ParseError::Int(ref e) => e.source(),
            ParseError::Float(ref e) => e.source(),
            ParseError::String(ref e) => e.source(),
            ParseError::Str(ref e) => e.source(),
            ParseError::Url(ref e) => e.source(),
            ParseError::Bincode(ref e) => e.source(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::Bool(ref e) => e.fmt(f),
            ParseError::Int(ref e) => e.fmt(f),
            ParseError::Float(ref e) => e.fmt(f),
            ParseError::String(ref e) => e.fmt(f),
            ParseError::Str(ref e) => e.fmt(f),
            ParseError::Url(ref e) => e.fmt(f),
            ParseError::Bincode(ref e) => e.fmt(f),
        }
    }
}

impl From<ParseError> for MemcachedError {
    fn from(err: ParseError) -> Self {
        MemcachedError::ParseError(err)
    }
}

impl From<string::FromUtf8Error> for MemcachedError {
    fn from(err: string::FromUtf8Error) -> MemcachedError {
        ParseError::String(err).into()
    }
}

impl From<str::Utf8Error> for MemcachedError {
    fn from(err: str::Utf8Error) -> MemcachedError {
        ParseError::Str(err).into()
    }
}

impl From<num::ParseIntError> for MemcachedError {
    fn from(err: num::ParseIntError) -> MemcachedError {
        ParseError::Int(err).into()
    }
}

impl From<num::ParseFloatError> for MemcachedError {
    fn from(err: num::ParseFloatError) -> MemcachedError {
        ParseError::Float(err).into()
    }
}

impl From<url::ParseError> for MemcachedError {
    fn from(err: url::ParseError) -> MemcachedError {
        ParseError::Url(err).into()
    }
}

impl From<str::ParseBoolError> for MemcachedError {
    fn from(err: str::ParseBoolError) -> MemcachedError {
        ParseError::Bool(err).into()
    }
}

/// Stands for errors raised from rust-memcache
#[derive(Debug)]
pub enum MemcachedError {
    /// Error raised when the provided memcache URL doesn't have a host name
    #[cfg(feature = "tls")]
    BadURL(String),
    /// `std::io` related errors.
    IOError(io::Error),
    /// Client Errors
    ClientError(ClientError),
    /// Server Errors
    ServerError(ServerError),
    /// Command specific Errors
    CommandError(CommandError),
    #[cfg(feature = "tls")]
    OpensslError(openssl::ssl::HandshakeError<std::net::TcpStream>),
    /// Parse errors
    ParseError(ParseError),
    /// pool error
    PoolError(&'static str),
}

impl fmt::Display for MemcachedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            #[cfg(feature = "tls")]
            MemcachedError::BadURL(ref s) => s.fmt(f),
            MemcachedError::IOError(ref err) => err.fmt(f),
            #[cfg(feature = "tls")]
            MemcachedError::OpensslError(ref err) => err.fmt(f),
            MemcachedError::ParseError(ref err) => err.fmt(f),
            MemcachedError::ClientError(ref err) => err.fmt(f),
            MemcachedError::ServerError(ref err) => err.fmt(f),
            MemcachedError::CommandError(ref err) => err.fmt(f),
            MemcachedError::PoolError(ref err) => err.fmt(f),
        }
    }
}

impl error::Error for MemcachedError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            #[cfg(feature = "tls")]
            MemcachedError::BadURL(_) => None,
            MemcachedError::IOError(ref err) => err.source(),
            #[cfg(feature = "tls")]
            MemcachedError::OpensslError(ref err) => err.source(),
            MemcachedError::ParseError(ref p) => p.source(),
            MemcachedError::ClientError(_)
            | MemcachedError::ServerError(_)
            | MemcachedError::CommandError(_)
            | MemcachedError::PoolError(_) => None,
        }
    }
}

impl From<io::Error> for MemcachedError {
    fn from(err: io::Error) -> MemcachedError {
        MemcachedError::IOError(err)
    }
}

impl<T> From<mobc::Error<T>> for MemcachedError {
    fn from(_: mobc::Error<T>) -> MemcachedError {
        MemcachedError::PoolError("mobc error")
    }
}

impl From<bincode::Error> for MemcachedError {
    fn from(e: bincode::Error) -> MemcachedError {
        MemcachedError::ParseError(ParseError::Bincode(e))
    }
}

#[cfg(feature = "tls")]
impl From<openssl::error::ErrorStack> for MemcachedError {
    fn from(err: openssl::error::ErrorStack) -> MemcachedError {
        MemcachedError::OpensslError(openssl::ssl::HandshakeError::<std::net::TcpStream>::from(
            err,
        ))
    }
}

#[cfg(feature = "tls")]
impl From<openssl::ssl::HandshakeError<std::net::TcpStream>> for MemcachedError {
    fn from(err: openssl::ssl::HandshakeError<std::net::TcpStream>) -> MemcachedError {
        MemcachedError::OpensslError(err)
    }
}
