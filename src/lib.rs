#[cfg(feature = "smol")]
use smol::io::{Error as SmolError, ErrorKind};

#[cfg(feature = "tokio")]
use tokio::io::{Error as TokioError, ErrorKind};

pub type XorResult<'x, T> = Result<T, XorError<'x>>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum XorError<'x> {
    Io(ErrorKind),
    FilePath(String),
    FilePathExt {
        cause: String,
        path: String,
    },
    UnsupportedImageFormat,
    FileSizeExceeded {
        capacity_allowed: u64,
        size_encountered: u64,
    },
    UnsupportedFormat(&'x str),
    UnsupportedStringEncoding(&'x str),
    UnsupportedBinaryEncoding(&'x str),
    UnsupportedDecodeString(&'x str),
    UnsupportedDecodeBinary(&'x str),
    #[cfg(feature = "base64")]
    Base64(Base64DecodeError),
    #[cfg(feature = "hex")]
    Hex(HexError),
    #[cfg(feature = "z85")]
    Z85(Z85DecodeError),
    #[cfg(feature = "lz4")]
    Lz4(Lz4Error),
}

#[cfg(feature = "smol")]
impl<'x> From<SmolError> for XorError<'x> {
    fn from(value: SmolError) -> Self {
        XorError::Io(value.kind())
    }
}

#[cfg(feature = "tokio")]
impl<'x> From<TokioError> for XorError<'x> {
    fn from(value: TokioError) -> Self {
        XorError::Io(value.kind())
    }
}

#[cfg(feature = "bas64")]
impl<'x> From<base64::DecodeError> for XorError<'x> {
    fn from(value: base64::DecodeError) -> Self {
        let error_value = match value {
            base64::DecodeError::InvalidByte(offset, invalid_byte) => {
                Base64DecodeError::InvalidByte(offset, invalid_byte)
            }
            base64::DecodeError::InvalidLength => Base64DecodeError::InvalidLength,
            base64::DecodeError::InvalidPadding => Base64DecodeError::InvalidPadding,
            base64::DecodeError::InvalidLastSymbol(offset, invalid_byte) => {
                Base64DecodeError::InvalidLastSymbol(offset, invalid_byte)
            }
        };

        XorError::Base64(error_value)
    }
}

#[cfg(feature = "hex")]
impl<'x> From<hex::FromHexError> for XorError<'x> {
    fn from(value: hex::FromHexError) -> Self {
        let error_value = match value {
            hex::FromHexError::InvalidHexCharacter { c, index } => {
                HexError::InvalidHexCharacter { c, index }
            }
            hex::FromHexError::OddLength => HexError::OddLength,
            hex::FromHexError::InvalidStringLength => HexError::InvalidStringLength,
        };

        XorError::Hex(error_value)
    }
}

#[cfg(feature = "z85")]
impl<'x> From<z85::DecodeError> for XorError<'x> {
    fn from(value: z85::DecodeError) -> Self {
        let error_value = match value {
            z85::DecodeError::InvalidByte(offset, invalid_byte) => {
                Z85DecodeError::InvalidByte(offset, invalid_byte)
            }
            z85::DecodeError::InvalidChunk(chunk) => Z85DecodeError::InvalidChunk(chunk),
            z85::DecodeError::InvalidLength(length) => Z85DecodeError::InvalidLength(length),
            z85::DecodeError::InvalidTail => Z85DecodeError::InvalidTail,
        };

        XorError::Z85(error_value)
    }
}

#[cfg(feature = "lz4")]
impl<'x> From<lz4_flex::block::DecompressError> for XorError<'x> {
    fn from(value: lz4_flex::block::DecompressError) -> Self {
        use lz4_flex::block::DecompressError;

        let error_value = match value {
            DecompressError::OutputTooSmall { expected, actual } => {
                Lz4Error::OutputTooSmall { expected, actual }
            }
            DecompressError::UncompressedSizeDiffers { expected, actual } => {
                Lz4Error::UncompressedSizeDiffers { expected, actual }
            }
            DecompressError::LiteralOutOfBounds => Lz4Error::LiteralOutOfBounds,
            DecompressError::ExpectedAnotherByte => Lz4Error::ExpectedAnotherByte,
            DecompressError::OffsetOutOfBounds => Lz4Error::OffsetOutOfBounds,
            _ => Lz4Error::UnsupportedError,
        };

        XorError::Lz4(error_value)
    }
}

#[cfg(feature = "base64")]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Base64DecodeError {
    /// An invalid byte was found in the input.
    /// The offset and offending byte are provided.
    /// Padding characters (=) interspersed in the encoded form will be treated as invalid bytes.
    InvalidByte(usize, u8),
    /// The length of the input is invalid. A typical cause of this is stray trailing whitespace or other separator bytes.
    /// In the case where excess trailing bytes have produced an invalid length and the last byte is also an invalid base64 symbol (as would be the case for whitespace, etc),
    /// InvalidByte will be emitted instead of InvalidLength to make the issue easier to debug.
    InvalidLength,
    /// The last non-padding input symbol’s encoded 6 bits have nonzero bits that will be discarded.
    /// This is indicative of corrupted or truncated Base64. Unlike InvalidByte, which reports symbols that aren’t in the alphabet,
    /// this error is for symbols that are in the alphabet but represent nonsensical encodings.
    InvalidLastSymbol(usize, u8),
    /// The nature of the padding was not as configured: absent or incorrect when it must be canonical, or present when it must be absent, etc.
    InvalidPadding,
}

#[cfg(feature = "hex")]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HexError {
    /// An invalid character was found. Valid ones are: 0...9, a...f or A...F.
    InvalidHexCharacter { c: char, index: usize },
    /// A hex string’s length needs to be even, as two digits correspond to one byte.
    OddLength,
    /// If the hex string is decoded into a fixed sized container, such as an array, the hex string’s length * 2 has to match the container’s length.
    InvalidStringLength,
}

#[cfg(feature = "z85")]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Z85DecodeError {
    InvalidByte(usize, u8),
    InvalidChunk(usize),
    InvalidLength(usize),
    InvalidTail,
}

#[cfg(feature = "lz4")]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lz4Error {
    OutputTooSmall { expected: usize, actual: usize },
    UncompressedSizeDiffers { expected: usize, actual: usize },
    LiteralOutOfBounds,
    ExpectedAnotherByte,
    OffsetOutOfBounds,
    UnsupportedError,
}
