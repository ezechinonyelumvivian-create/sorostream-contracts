use soroban_sdk::{contractclient, Env};

/// Standard interface for contracts that consume streaming payments from SoroStream.
#[contractclient(name = "StreamConsumerClient")]
pub trait StreamConsumer {
    /// Called by SoroStream when tokens are withdrawn into this contract.
    /// 
    /// # Arguments
    /// * `env` - The contract environment.
    /// * `stream_id` - The ID of the stream from which tokens were withdrawn.
    /// * `amount` - The amount of tokens withdrawn (in stroops).
    fn on_stream_withdraw(env: Env, stream_id: u64, amount: i128);
}
