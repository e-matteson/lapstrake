use std::error::Error;
use std::fmt;
use std::io;

use csv;

use scad_dots::errors::ScadDotsError;

#[derive(Debug)]
pub enum LapstrakeError {
    General(String),
    Load,
    Spline,
    Draw,
    Model(ScadDotsError),
    Io(io::Error),
    Csv(csv::Error),
    Context {
        message: String,
        cause: Box<LapstrakeError>,
    },
}

pub trait ResultExt<T> {
    /// Convert the error type to a LapstrakeError, and add the context message around it.
    fn context(self, message: &str) -> Result<T, LapstrakeError>;

    /// Like `context()` but take a closure containing a potentionally costly
    /// operation that will only be executed if there was an error.
    fn with_context<F>(self, message_creator: F) -> Result<T, LapstrakeError>
    where
        F: Fn() -> String;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    LapstrakeError: From<E>,
{
    fn context(self, message: &str) -> Result<T, LapstrakeError> {
        self.map_err(|err| LapstrakeError::from(err).context(message))
    }

    fn with_context<F>(self, message_creator: F) -> Result<T, LapstrakeError>
    where
        F: Fn() -> String,
    {
        self.context(&message_creator())
    }
}

impl LapstrakeError {
    /// Wrap the error with a message providing more context about what went wrong.
    pub fn context(self, message: &str) -> Self {
        LapstrakeError::Context {
            message: message.to_owned(),
            cause: Box::new(self),
        }
    }

    /// Like `context()` but take a closure containing a potentionally costly
    /// operation that will only be executed if there was an error.
    pub fn with_context<T>(self, message_creator: T) -> Self
    where
        T: Fn() -> String,
    {
        self.context(&message_creator())
    }
}

impl Error for LapstrakeError {
    fn cause(&self) -> Option<&Error> {
        match self {
            LapstrakeError::Context { ref cause, .. } => Some(cause),
            LapstrakeError::Model(ref cause) => Some(cause),
            LapstrakeError::Io(ref cause) => Some(cause),
            _ => None,
        }
    }
}

impl From<io::Error> for LapstrakeError {
    fn from(io_err: io::Error) -> LapstrakeError {
        LapstrakeError::Io(io_err)
    }
}

impl From<ScadDotsError> for LapstrakeError {
    fn from(scad_err: ScadDotsError) -> LapstrakeError {
        LapstrakeError::Model(scad_err)
    }
}

impl From<csv::Error> for LapstrakeError {
    fn from(csv_err: csv::Error) -> LapstrakeError {
        LapstrakeError::Csv(csv_err)
    }
}

impl fmt::Display for LapstrakeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LapstrakeError::Load => {
                write!(f, "Failed to load ship specification")
            }
            LapstrakeError::Draw => write!(f, "Failed to make 2d drawing"),
            LapstrakeError::Spline => write!(f, "Spline error"),
            LapstrakeError::Io(err) => write!(f, "Input/output error: {}", err),
            LapstrakeError::Csv(err) => write!(f, "CSV file error: {}", err),
            LapstrakeError::Model(err) => {
                write!(f, "Failed to make 3d model: {}", err)
            }
            LapstrakeError::General(message) => write!(f, "{}", message),
            LapstrakeError::Context { message, cause } => {
                write!(f, "{}\n  caused by: {}", message, cause)
            }
        }
    }
}
