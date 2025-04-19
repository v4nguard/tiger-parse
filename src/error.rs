use std::fmt::Display;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("while reading {stack}: {error}")]
    PropagatedError {
        stack: FieldRecordStack,
        error: Box<Error>,
    },

    #[error("Type mismatch: {0}")]
    TypeMismatch(String),

    #[error("Padding bytes are not zero! Got {0:X?}")]
    PaddingNotZero(Vec<u8>),

    #[error("String too long")]
    StringTooLong,

    #[error("Pointer is null")]
    PointerNull,

    #[cfg(feature = "tiger_pkg")]
    #[error("Hash64 lookup failed for {0}")]
    Hash64LookupFailed(tiger_pkg::TagHash64),

    #[cfg(feature = "tiger_pkg")]
    #[error("Tag read failed: {0}")]
    TagReadFailed(String),
}

/// Represents a field in a propagated error, eg. `User.name`
#[derive(Debug)]
pub struct FieldRecord {
    pub typename: String,
    pub field: String,
}

impl Display for FieldRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}.{}", self.typename, self.field))
    }
}

/// Represents a stack of fields in a propagated error, eg. `[User.authentication -> Authentication.password]`
#[derive(Debug, Default)]
pub struct FieldRecordStack(Vec<FieldRecord>);

impl FieldRecordStack {
    pub fn push_front(&mut self, record: FieldRecord) {
        self.0.insert(0, record);
    }
}

impl Display for FieldRecordStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        for (i, record) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(" -> ")?;
            }
            record.fmt(f)?;
        }
        f.write_str("]")
    }
}

pub trait ResultExt<T> {
    fn with_field(self, typename: &str, field: &str) -> Result<T, Error>;
}

impl<T> ResultExt<T> for Result<T, Error> {
    fn with_field(self, typename: &str, field: &str) -> Result<T, Error> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => match error {
                // Add to existing propagated error
                Error::PropagatedError { mut stack, error } => {
                    stack.push_front(FieldRecord {
                        typename: typename.to_string(),
                        field: field.to_string(),
                    });

                    Err(Error::PropagatedError { stack, error })
                }
                // New propagated error
                e => Err(Error::PropagatedError {
                    stack: FieldRecordStack(vec![FieldRecord {
                        typename: typename.to_string(),
                        field: field.to_string(),
                    }]),
                    error: Box::new(e),
                }),
            },
        }
    }
}
