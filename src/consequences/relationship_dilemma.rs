//! Emergent tenant dilemma: a tenant paying premium rent whose hostile
//! relationships are dragging down their neighbors. Surfaces a keep-or-evict
//! choice event built from the `dilemma` templates in relationship_events.json.
//!
//! Placeholder conventions for dilemma templates:
//! - Text: `{tenant}` / `{apt}` = the disruptor, `{rent}` their monthly rent,
//!   `{victim_count}` / `{victims}` the affected neighbors.
//! - Effects: `tenant_id` 0 = the disruptor, 1 = expanded to every victim;
//!   `RelationshipStrength` with placeholder ids expands to every
//!   disruptor-victim pair.

use crate::building::Building;
use crate::consequences::relationships::{RelationshipType, TenantNetwork};
use crate::data::config::DilemmaConfig;
use crate::narrative::events::{NarrativeChoice, NarrativeEffect, NarrativeEventType};
use crate::narrative::relationship_config::RelationshipEventTemplate;
use crate::narrative::{NarrativeEvent, RelationshipEventsConfig};
use crate::tenant::Tenant;
use macroquad_toolkit::rng;

/// A tenant who qualifies for the keep-or-evict dilemma
pub struct DisruptorInfo {
    pub tenant_id: u32,
    pub victim_ids: Vec<u32>,
    pub rent: i32,
}

impl TenantNetwork {
    /// Roll for an emergent dilemma event this month. At most one fires, and a
    /// given tenant can only star in one every `cooldown_months`.
    pub(crate) fn maybe_generate_dilemma(
        &mut self,
        tenants: &[Tenant],
        building: &Building,
        config: &DilemmaConfig,
        events_config: &RelationshipEventsConfig,
        current_month: u32,
    ) -> Option<NarrativeEvent> {
        if events_config.dilemma.is_empty() {
            return None;
        }
        let info = self.find_disruptor(tenants, building, config, current_month)?;
        for template in &events_config.dilemma {
            if rng::gen_range(0, 100) >= template.probability as i32 {
                continue;
            }
            if let Some(event) = generate_dilemma_event(template, &info, tenants, building) {
                self.dilemma_history.insert(info.tenant_id, current_month);
                return Some(event);
            }
        }
        None
    }

    /// Find the highest-rent tenant with enough hostile relationships (or one
    /// hostile relationship plus poor behavior) whose rent sits above the
    /// building average — the money-vs-community tension is what makes it a
    /// dilemma rather than an obvious eviction.
    fn find_disruptor(
        &self,
        tenants: &[Tenant],
        building: &Building,
        config: &DilemmaConfig,
        current_month: u32,
    ) -> Option<DisruptorInfo> {
        let occupied_rents: Vec<i32> = building
            .apartments
            .iter()
            .filter(|a| !a.is_vacant())
            .map(|a| a.rent_price)
            .collect();
        if occupied_rents.is_empty() {
            return None;
        }
        let avg_rent = occupied_rents.iter().sum::<i32>() as f32 / occupied_rents.len() as f32;
        let rent_floor = avg_rent * config.rent_premium_multiplier;

        let mut best: Option<DisruptorInfo> = None;
        for tenant in tenants {
            if let Some(fired) = self.dilemma_history.get(&tenant.id) {
                if current_month.saturating_sub(*fired) < config.cooldown_months {
                    continue;
                }
            }
            let rent = match tenant
                .apartment_id
                .and_then(|id| building.get_apartment(id))
            {
                Some(apt) => apt.rent_price,
                None => continue,
            };
            if (rent as f32) < rent_floor {
                continue;
            }

            let victim_ids: Vec<u32> = self
                .relationships
                .iter()
                .filter(|r| matches!(r.relationship_type, RelationshipType::Hostile))
                .filter_map(|r| {
                    if r.tenant_a_id == tenant.id {
                        Some(r.tenant_b_id)
                    } else if r.tenant_b_id == tenant.id {
                        Some(r.tenant_a_id)
                    } else {
                        None
                    }
                })
                .filter(|id| tenants.iter().any(|t| t.id == *id))
                .collect();

            let qualifies = victim_ids.len() >= config.min_hostile_relationships as usize
                || (!victim_ids.is_empty()
                    && tenant.behavior_score < config.single_hostile_max_behavior);
            if !qualifies {
                continue;
            }

            if best.as_ref().is_none_or(|b| rent > b.rent) {
                best = Some(DisruptorInfo {
                    tenant_id: tenant.id,
                    victim_ids,
                    rent,
                });
            }
        }
        best
    }
}

fn generate_dilemma_event(
    template: &RelationshipEventTemplate,
    info: &DisruptorInfo,
    tenants: &[Tenant],
    building: &Building,
) -> Option<NarrativeEvent> {
    let disruptor = tenants.iter().find(|t| t.id == info.tenant_id)?;
    let apt = disruptor
        .apartment_id
        .and_then(|id| building.get_apartment(id))
        .map(|a| format!("Apt {}", a.unit_number))
        .unwrap_or("Unknown".to_string());
    let victim_names: Vec<&str> = info
        .victim_ids
        .iter()
        .filter_map(|id| tenants.iter().find(|t| t.id == *id))
        .map(|t| t.name.as_str())
        .collect();

    let substitute = |text: &str| {
        text.replace("{tenant}", &disruptor.name)
            .replace("{apt}", &apt)
            .replace("{rent}", &info.rent.to_string())
            .replace("{victim_count}", &info.victim_ids.len().to_string())
            .replace("{victims}", &victim_names.join(", "))
    };

    let choices: Vec<NarrativeChoice> = template
        .choices
        .iter()
        .map(|c| NarrativeChoice {
            label: substitute(&c.label),
            description: substitute(&c.description),
            effect: resolve_dilemma_effect(&c.effect, info),
            reputation_change: c.reputation_change,
        })
        .collect();

    let default_effect = template
        .default_effect
        .as_ref()
        .map(|e| resolve_dilemma_effect(e, info))
        .unwrap_or(NarrativeEffect::None);

    Some(NarrativeEvent {
        id: 0,    // Set by NarrativeEventSystem
        month: 0, // Set by caller
        event_type: NarrativeEventType::RelationshipEvent,
        headline: substitute(&template.headline),
        description: substitute(&template.description),
        choices,
        default_effect,
        read: false,
        requires_response: !template.choices.is_empty(),
        response_deadline: if !template.choices.is_empty() {
            Some(2)
        } else {
            None
        },
        related_neighborhood_id: None,
    })
}

/// Expand placeholder tenant ids: 0 = disruptor, 1 = every victim (fanned out
/// into a Multiple), relationship pairs = every disruptor-victim pair.
fn resolve_dilemma_effect(effect: &NarrativeEffect, info: &DisruptorInfo) -> NarrativeEffect {
    let fan_out = |make: &dyn Fn(u32) -> NarrativeEffect| {
        let effects: Vec<NarrativeEffect> = info.victim_ids.iter().map(|v| make(*v)).collect();
        match effects.len() {
            1 => effects.into_iter().next().unwrap(),
            _ => NarrativeEffect::Multiple { effects },
        }
    };

    match effect {
        NarrativeEffect::TenantHappiness { tenant_id, change } => match tenant_id {
            0 => NarrativeEffect::TenantHappiness {
                tenant_id: info.tenant_id,
                change: *change,
            },
            1 => fan_out(&|v| NarrativeEffect::TenantHappiness {
                tenant_id: v,
                change: *change,
            }),
            _ => effect.clone(),
        },
        NarrativeEffect::OpinionChange { tenant_id, amount } => match tenant_id {
            0 => NarrativeEffect::OpinionChange {
                tenant_id: info.tenant_id,
                amount: *amount,
            },
            1 => fan_out(&|v| NarrativeEffect::OpinionChange {
                tenant_id: v,
                amount: *amount,
            }),
            _ => effect.clone(),
        },
        NarrativeEffect::MoveOut { tenant_id } => match tenant_id {
            0 => NarrativeEffect::MoveOut {
                tenant_id: info.tenant_id,
            },
            1 => fan_out(&|v| NarrativeEffect::MoveOut { tenant_id: v }),
            _ => effect.clone(),
        },
        NarrativeEffect::RelationshipStrength { change, .. } => {
            let disruptor = info.tenant_id;
            fan_out(&|v| NarrativeEffect::RelationshipStrength {
                tenant_a_id: disruptor,
                tenant_b_id: v,
                change: *change,
            })
        }
        NarrativeEffect::Multiple { effects } => NarrativeEffect::Multiple {
            effects: effects
                .iter()
                .map(|e| resolve_dilemma_effect(e, info))
                .collect(),
        },
        _ => effect.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::building::Building;
    use crate::tenant::TenantArchetype;

    fn make_building() -> Building {
        Building::new("Test", 2, 2)
    }

    fn place_tenant(id: u32, name: &str, building: &mut Building, apt_index: usize) -> Tenant {
        let mut tenant = Tenant::new(id, name, TenantArchetype::Professional);
        let apt = &mut building.apartments[apt_index];
        apt.tenant_id = Some(id);
        tenant.apartment_id = Some(apt.id);
        tenant
    }

    fn setup() -> (TenantNetwork, Vec<Tenant>, Building, DilemmaConfig) {
        let mut building = make_building();
        for apt in &mut building.apartments {
            apt.rent_price = 500;
        }
        let sarah = place_tenant(1, "Sarah", &mut building, 0);
        let alex = place_tenant(2, "Alex", &mut building, 1);
        let kim = place_tenant(3, "Kim", &mut building, 2);
        building.apartments[0].rent_price = 900; // Sarah pays a premium

        let mut network = TenantNetwork::new();
        network.apply_relationship_change(1, 2, -40); // Sarah-Alex hostile
        network.apply_relationship_change(1, 3, -40); // Sarah-Kim hostile

        (
            network,
            vec![sarah, alex, kim],
            building,
            DilemmaConfig::default(),
        )
    }

    #[test]
    fn detects_high_rent_disruptor() {
        let (network, tenants, building, config) = setup();
        let info = network
            .find_disruptor(&tenants, &building, &config, 10)
            .expect("Sarah should qualify");
        assert_eq!(info.tenant_id, 1);
        assert_eq!(info.rent, 900);
        let mut victims = info.victim_ids.clone();
        victims.sort();
        assert_eq!(victims, vec![2, 3]);
    }

    #[test]
    fn average_rent_troublemaker_is_not_a_dilemma() {
        let (network, tenants, mut building, config) = setup();
        building.apartments[0].rent_price = 500; // No premium, no dilemma
        assert!(network
            .find_disruptor(&tenants, &building, &config, 10)
            .is_none());
    }

    #[test]
    fn cooldown_suppresses_repeat_dilemmas() {
        let (mut network, tenants, building, config) = setup();
        network.dilemma_history.insert(1, 8);
        assert!(network
            .find_disruptor(&tenants, &building, &config, 10)
            .is_none());
        assert!(network
            .find_disruptor(&tenants, &building, &config, 8 + config.cooldown_months)
            .is_some());
    }

    #[test]
    fn placeholders_substitute_and_effects_resolve() {
        let (network, tenants, building, config) = setup();
        let info = network
            .find_disruptor(&tenants, &building, &config, 10)
            .unwrap();

        let template = RelationshipEventTemplate {
            id: "test".to_string(),
            trigger_strength_min: None,
            trigger_strength_max: None,
            probability: 100,
            headline: "The {tenant} Problem".to_string(),
            description: "{tenant} in {apt} pays ${rent}, {victim_count} suffer: {victims}"
                .to_string(),
            choices: vec![
                crate::narrative::relationship_config::RelationshipChoiceTemplate {
                    label: "Evict {tenant}".to_string(),
                    description: String::new(),
                    effect: NarrativeEffect::Multiple {
                        effects: vec![
                            NarrativeEffect::MoveOut { tenant_id: 0 },
                            NarrativeEffect::TenantHappiness {
                                tenant_id: 1,
                                change: 20,
                            },
                        ],
                    },
                    reputation_change: 0,
                },
            ],
            default_effect: None,
        };

        let event = generate_dilemma_event(&template, &info, &tenants, &building).unwrap();
        assert_eq!(event.headline, "The Sarah Problem");
        assert!(event.description.contains("$900"));
        assert!(event.description.contains("2 suffer"));
        assert!(event.description.contains("Alex"));
        assert!(event.requires_response);
        assert_eq!(event.choices[0].label, "Evict Sarah");

        match &event.choices[0].effect {
            NarrativeEffect::Multiple { effects } => {
                assert!(matches!(
                    effects[0],
                    NarrativeEffect::MoveOut { tenant_id: 1 }
                ));
                // Victim happiness fans out to both hostile neighbors
                match &effects[1] {
                    NarrativeEffect::Multiple { effects: fanned } => {
                        assert_eq!(fanned.len(), 2);
                    }
                    other => panic!("expected fan-out, got {:?}", other),
                }
            }
            other => panic!("expected Multiple, got {:?}", other),
        }
    }
}
