mod tick;
mod decay;
mod events;
mod win_condition;

pub use tick::{GameTick, TickResult, advance_tick};
pub use decay::{apply_decay, turns_until_critical, assess_building_health, BuildingHealth, rates};
pub use events::{GameEvent, EventLog, EventSeverity};
pub use win_condition::{check_win_condition, get_victory_progress, GameOutcome, VictoryProgress, thresholds};
