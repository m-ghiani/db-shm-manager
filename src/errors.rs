#[derive(Debug)]
pub enum DbShmError {
    InvalidSize(usize, usize), // Aggiunto: dimensione attesa, dimensione reale
    SerializationError(String, usize, usize), // Aggiunto: messaggio di errore, dimensione attesa, dimensione reale
    DeserializationError(String, usize, usize), // Aggiunto: messaggio di errore, dimensione attesa, dimensione reale
}

impl std::fmt::Display for DbShmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbShmError::InvalidSize(expected, actual) => {
                write!(f, "Invalid size provided. Expected: {}, Actual: {}", expected, actual)
            },
            DbShmError::SerializationError(msg, expected, actual) => {
                write!(f, "Serialization error: {}. Expected size: {}, Actual size: {}", msg, expected, actual)
            },
            DbShmError::DeserializationError(msg, expected, actual) => {
                write!(f, "Deserialization error: {}. Expected size: {}, Actual size: {}", msg, expected, actual)
            },
        }
    }
}

impl std::error::Error for DbShmError {}
