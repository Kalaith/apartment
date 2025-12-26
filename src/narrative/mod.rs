//! # Narrative Module
//! 
//! Handles story-driven elements and dynamic events:
//! - `Stories`: Procedural tenant storylines.
//! - `Events`: One-off or chained narrative events.
//! - `Mail`: In-game messaging system.
//! - `Tutorial`: Guided introduction flow.
//! - `Missions`: Quests and objectives.

mod stories;
mod events;
mod mail;
mod tutorial;
mod missions;
pub mod dialogue;  // Make public so DialogueEffect is accessible

pub use stories::{TenantStory, StoryImpact};
pub use events::NarrativeEventSystem;
pub use mail::Mailbox;
pub use tutorial::{TutorialManager, TutorialMilestone};
pub use missions::{MissionManager, MissionGoal, MissionReward, LegacyEvent, BuildingAward, MissionStatus};
pub use dialogue::DialogueSystem;

