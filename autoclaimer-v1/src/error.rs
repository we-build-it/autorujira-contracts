// src/error.rs
use cosmwasm_std::StdError;
use serde_json::Error as SerdeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Owner should be specified")]
    NoOwner,

    #[error("You have no permissions to execute this function")]
    Unauthorized,

    #[error("No rewards available in the contract. Message: {msg:?}")]
    NoRewards { msg: String },

    #[error("Invalid reply ID: {id}")]
    InvalidReplyId { id: u64 },

    // Agrega otros errores personalizados si es necesario
    #[error("Serialization error: {0}")]
    SerializationError(String), // Nuevo error para manejo de serialización

    #[error("Too many protocols to claim: {max_allowed}")]
    TooManyMessages { max_allowed: usize },

    #[error("Unsupported protocol: {protocol}")]
    InvalidProtocol { protocol: String },
}

// Implementación de From<serde_json::Error> para ContractError
impl From<SerdeError> for ContractError {
    fn from(e: SerdeError) -> Self {
        ContractError::SerializationError(e.to_string())
    }
}
