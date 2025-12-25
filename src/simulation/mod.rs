mod tick;
mod decay;
mod events;
mod win_condition;
mod random_events;

pub use tick::{TickResult, advance_tick};
// pub use decay::apply_decay;
pub use events::{GameEvent, EventLog, EventSeverity};
pub use win_condition::GameOutcome;
pub use random_events::EventSystem;
