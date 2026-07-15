//! Emergent tenant life events — the ongoing pressure that turns "who lives
//! here" into a story. Each month a tenant may experience a life change (new
//! job, job loss, new baby, retirement…) that applies concrete gameplay
//! consequences and surfaces to the player.

use macroquad::prelude::*;
use macroquad_toolkit::rng;

use crate::narrative::{LifeChangeType, StoryImpact};
use crate::simulation::{GameEvent, NotificationLevel};
use crate::tenant::TenantArchetype;
use crate::ui::colors;

use super::gameplay::GameplayState;

impl GameplayState {
    /// Roll monthly life events for the current tenants. Frequency and impact
    /// magnitudes are data-driven (`config.life_events`); the archetype→event
    /// eligibility and the mapping to concrete consequences live in
    /// [`LifeChangeType`].
    pub(super) fn generate_tenant_life_events(&mut self) {
        let chance = self.config.life_events.monthly_chance_percent;
        if chance <= 0 || self.tenants.is_empty() {
            return;
        }

        // Snapshot the affected tenants first so we can mutate state freely while
        // applying each impact.
        let struck: Vec<(u32, String, TenantArchetype)> = self
            .tenants
            .iter()
            .filter(|_| rng::gen_range(0, 100) < chance)
            .map(|t| (t.id, t.name.clone(), t.archetype.clone()))
            .collect();

        for (tenant_id, name, archetype) in struck {
            let options = LifeChangeType::eligible_for(&archetype);
            let Some(change) = rng::choose(&options).cloned() else {
                continue;
            };
            let (_impact, description) = change.impact(&self.config.life_events);

            // Apply the concrete consequences through the story-impact pipeline
            // (which expands the LifeChange into happiness / rent-tolerance /
            // move-out-risk effects).
            self.apply_story_impact(tenant_id, StoryImpact::LifeChange(change.clone()));

            // Record it on the tenant's story for history, when one exists.
            if let Some(story) = self.tenant_stories.get_mut(&tenant_id) {
                story.add_event(
                    self.current_tick,
                    &format!("{} {}", name, description),
                    StoryImpact::LifeChange(change),
                );
            }

            // Surface the emergent story to the player.
            self.event_log.log(
                GameEvent::Notification {
                    message: format!("{} {}.", name, description),
                    level: NotificationLevel::Info,
                },
                self.current_tick,
            );
            self.floating_texts.spawn(
                format!("{}: {}", name, description),
                vec2(screen_width() / 2.0, screen_height() / 2.0 - 40.0),
                colors::ACCENT(),
            );
        }
    }
}
