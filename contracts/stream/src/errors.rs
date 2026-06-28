use soroban_sdk::contracterror;

/// Custom errors for the SoroStream contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StreamError {
    StreamNotFound = 1,
    NotRecipient = 2,
    NotSender = 3,
    StreamNotActive = 4,
    ZeroAmount = 5,
    InvalidDuration = 6,
    InsufficientBalance = 7,
    InvalidCliff = 8,
    AlreadyInitialized = 9,
    NotInitialized = 10,
    DuplicateStream = 11,
    InvalidStartTime = 12,
    InvalidPartialCancel = 13,
    ContractPaused = 14,
    Overflow = 15,
    ZeroFlowRate = 16,
    BatchLengthMismatch = 17,
    TokenMismatch = 18,
    /// Caller is not authorized.
    NotAuthorized = 20,
    /// Stream is not paused.
    StreamNotPaused = 21,
    /// Stream is locked until the lock_until timestamp.
    StreamLocked = 19,
    StreamLocked = 19,
    NotAuthorized = 20,
    StreamNotPaused = 21,
    StreamDurationTooShort = 22,
}
