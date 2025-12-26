//! # Simulation Module
//! 
//! The heartbeat of the game. Handles time and state progression:
//! - `Tick`: The central game loop processing logic.
//! - `Decay`: Entropy and maintenance mechanics.
//! - `Win Conditions`: Victory and failure state checks.
//! - `Events`: Random events and lucky/unlucky occurrences.

mod tick;
mod decay;
mod events;
mod win_condition;
mod random_events;

pub use tick::{TickResult, advance_tick};
// pub use decay::apply_decay;
pub use events::{GameEvent, EventLog, EventSeverity, NotificationLevel};
pub use win_condition::GameOutcome;
pub use random_events::EventSystem;
