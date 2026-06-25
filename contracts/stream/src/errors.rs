use soroban_sdk::contracterror;

/// Custom errors for the SoroStream contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StreamError {
    /// No stream exists with the given ID.
    StreamNotFound = 1,
    /// Caller is not the stream recipient.
    NotRecipient = 2,
    /// Caller is not the stream sender.
    NotSender = 3,
    /// Stream is not in Active status.
    StreamNotActive = 4,
    /// Amount must be greater than zero.
    ZeroAmount = 5,
    /// Duration must be greater than zero.
    InvalidDuration = 6,
    /// Contract has insufficient token balance.
    InsufficientBalance = 7,
    /// cancel_amount exceeds the unstreamed remainder or leaves too little to form a new stream.
    InvalidPartialCancel = 8,
}
