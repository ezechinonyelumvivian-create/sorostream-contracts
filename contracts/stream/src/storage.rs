use soroban_sdk::{Address, Env, Symbol, Vec};
use crate::types::Stream;

const STREAM_ID_KEY: &str = "next_id";

/// Returns and increments the global stream ID counter.
pub fn next_stream_id(env: &Env) -> u64 {
    let id: u64 = env.storage().instance().get(&Symbol::new(env, STREAM_ID_KEY)).unwrap_or(0u64);
    env.storage().instance().set(&Symbol::new(env, STREAM_ID_KEY), &(id + 1));
    id
}

/// Persists a stream to storage.
pub fn save_stream(env: &Env, stream: &Stream) {
    env.storage().persistent().set(&stream.id, stream);
}

/// Loads a stream from storage. Returns None if not found.
pub fn load_stream(env: &Env, stream_id: u64) -> Option<Stream> {
    env.storage().persistent().get(&stream_id)
}

/// Appends a stream ID to the sender's index.
pub fn index_by_sender(env: &Env, sender: &Address, stream_id: u64) {
    let key = (Symbol::new(env, "s"), sender.clone());
    let mut ids: Vec<u64> = env.storage().temporary().get(&key).unwrap_or(Vec::new(env));
    ids.push_back(stream_id);
    env.storage().temporary().set(&key, &ids);
}

/// Appends a stream ID to the recipient's index.
pub fn index_by_recipient(env: &Env, recipient: &Address, stream_id: u64) {
    let key = (Symbol::new(env, "r"), recipient.clone());
    let mut ids: Vec<u64> = env.storage().temporary().get(&key).unwrap_or(Vec::new(env));
    ids.push_back(stream_id);
    env.storage().temporary().set(&key, &ids);
}

/// Returns all stream IDs for a sender.
pub fn get_ids_by_sender(env: &Env, sender: &Address) -> Vec<u64> {
    let key = (Symbol::new(env, "s"), sender.clone());
    env.storage().temporary().get(&key).unwrap_or(Vec::new(env))
}

/// Returns all stream IDs for a recipient.
pub fn get_ids_by_recipient(env: &Env, recipient: &Address) -> Vec<u64> {
    let key = (Symbol::new(env, "r"), recipient.clone());
    env.storage().temporary().get(&key).unwrap_or(Vec::new(env))
}

/// Returns true if this (sender, nonce) pair has already been used.
pub fn nonce_used(env: &Env, sender: &Address, nonce: u64) -> bool {
    let key = (Symbol::new(env, "n"), sender.clone(), nonce);
    env.storage().persistent().has(&key)
}

/// Records a (sender, nonce) pair as used.
pub fn mark_nonce_used(env: &Env, sender: &Address, nonce: u64) {
    let key = (Symbol::new(env, "n"), sender.clone(), nonce);
    env.storage().persistent().set(&key, &true);
}
