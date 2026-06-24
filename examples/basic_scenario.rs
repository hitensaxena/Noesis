//! Basic scenario: start noesis, inject an experience, observe the cascade.
//!
//! Run with: cargo run --release -- inject "I went for a run in the park today"
//!
//! Expected output:
//!   - IngestRequest is published
//!   - EpisodeProcessor converts it to EpisodeRecorded
//!   - NarrativeProcessor may generate a narrative (every 3 episodes)
//!   - CuriosityProcessor may detect a knowledge gap (every 5 episodes)
//!   - The cascade eventually reaches equilibrium
//!
//! To start the daemon:
//!   cargo run -- start
//!
//! To start with REST API:
//!   cargo run -- start --rest

fn main() {
    println!("Run this scenario using cargo run:");
    println!("  cargo run -- start");
    println!("  cargo run -- inject 'Your experience here'");
    println!("  cargo run -- list all");
}
