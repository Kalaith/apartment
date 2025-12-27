//! # Narrative Module
//! 
//! Handles story-driven elements and dynamic events:
//! - `Stories`: Procedural tenant storylines.
//! - `Events`: One-off or chained narrative events.
//! - `Mail`: In-game messaging system.
//! - `Tutorial`: Guided introduction flow.
//! - `Missions`: Quests and objectives.
//! - `Notifications`: Game hints and relationship change pop-ups.

mod stories;
pub mod events;
mod mail;
mod tutorial;
mod missions;
pub mod dialogue;  // Make public so DialogueEffect is accessible
pub mod notifications;

pub use stories::{TenantStory, StoryImpact, TenantRequest};
pub use events::{NarrativeEventSystem, NarrativeEvent};
pub use mail::Mailbox;
pub use tutorial::{TutorialManager, TutorialMilestone};
pub use missions::{MissionManager, MissionGoal, MissionReward, MissionStatus};
pub use dialogue::DialogueSystem;
pub use notifications::{NotificationManager, NotificationCategory, RelationshipChange};
pub mod achievements;
pub use achievements::AchievementSystem;
pub mod events_config;
pub mod relationship_config;
pub use events_config::{TenantEventsConfig, load_events_config};
pub use relationship_config::{RelationshipEventsConfig, load_relationship_config};

