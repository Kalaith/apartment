use macroquad_toolkit::rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DialogueType {
    /// High-priority tenant issue (broken heater, pest infestation)
    FaceToFaceRequest,
    /// Tenant A complaining about Tenant B
    ConflictMediation,
    /// Rent change conversations
    RentNegotiation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DialogueEffect {
    /// Change tenant happiness
    HappinessChange { tenant_id: u32, amount: i32 },
    /// Gain or lose money
    MoneyChange(i32),
    /// Change tension between apartments
    TensionChange { apt_a: u32, apt_b: u32, amount: i32 },
    /// Change relationship between tenants
    RelationshipChange {
        tenant_a: u32,
        tenant_b: u32,
        change: i32,
    },
    /// Change landlord opinion
    OpinionChange { tenant_id: u32, amount: i32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DialogueChoice {
    pub text: String,
    pub effects: Vec<DialogueEffect>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveDialogue {
    pub id: u32,
    pub dialogue_type: DialogueType,
    pub initiator_id: u32,
    /// Other tenant involved (if conflict)
    pub target_id: Option<u32>,
    pub headline: String,
    pub description: String,
    pub choices: Vec<DialogueChoice>,
    /// When auto-resolves (if ignored)
    pub deadline_month: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DialogueSystem {
    pub active_dialogues: Vec<ActiveDialogue>,
    next_id: u32,
}

impl DialogueSystem {
    pub fn new() -> Self {
        Self {
            active_dialogues: Vec::new(),
            next_id: 1,
        }
    }

    /// Queue a new dialogue
    pub fn add_dialogue(
        &mut self,
        dialogue_type: DialogueType,
        initiator: u32,
        target: Option<u32>,
        headline: &str,
        description: &str,
        choices: Vec<DialogueChoice>,
        deadline: Option<u32>,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        self.active_dialogues.push(ActiveDialogue {
            id,
            dialogue_type,
            initiator_id: initiator,
            target_id: target,
            headline: headline.to_string(),
            description: description.to_string(),
            choices,
            deadline_month: deadline,
        });

        id
    }

    /// Get dialogues needing response
    pub fn pending_dialogues(&self) -> Vec<&ActiveDialogue> {
        self.active_dialogues.iter().collect()
    }

    /// Apply selected choice and return effects
    pub fn resolve_dialogue(
        &mut self,
        dialogue_id: u32,
        choice_index: usize,
    ) -> Option<Vec<DialogueEffect>> {
        if let Some(index) = self
            .active_dialogues
            .iter()
            .position(|d| d.id == dialogue_id)
        {
            let dialogue = self.active_dialogues.remove(index);

            if let Some(choice) = dialogue.choices.get(choice_index) {
                return Some(choice.effects.clone());
            }
        }
        None
    }

    /// Generate dialogues based on game state
    pub fn generate_dialogues(
        &mut self,
        month: u32,
        tenants: &[crate::tenant::Tenant],
        building: &crate::building::Building,
        funds: &crate::economy::PlayerFunds,
        network: &crate::consequences::TenantNetwork,
    ) {
        self.generate_conflict_mediation(tenants, network);
        self.generate_rent_negotiations(building, tenants);

        // Low funds can trigger rent negotiation dialogues
        let is_low_on_funds = funds.balance < 500;
        let building_quality = building.building_appeal();

        // Small chance for random face-to-face request from unhappy tenants
        for tenant in tenants {
            // More likely to complain if building quality is low or it's been many months
            let complaint_chance = if building_quality < 50 { 10 } else { 5 };
            let months_factor = if month > 12 { 2 } else { 0 };

            if tenant.happiness < 40 && rng::gen_range(0, 100) < (complaint_chance + months_factor)
            {
                // Determine request type based on archetype
                let (headline, desc, choices) = match tenant.archetype {
                    crate::tenant::TenantArchetype::Student => (
                        "Party Permission",
                        "I'm planning a small... study group. Might get a bit loud. Is that cool?",
                        vec![
                            DialogueChoice {
                                text: "Sure, just keep it down after 10pm".to_string(),
                                effects: vec![DialogueEffect::HappinessChange {
                                    tenant_id: tenant.id,
                                    amount: 5,
                                }],
                            },
                            DialogueChoice {
                                text: "Absolutely not".to_string(),
                                effects: vec![DialogueEffect::HappinessChange {
                                    tenant_id: tenant.id,
                                    amount: -5,
                                }],
                            },
                        ],
                    ),
                    _ => {
                        // Adjust repair cost based on landlord's funds
                        let repair_cost = if is_low_on_funds { 30 } else { 50 };
                        (
                            "Minor Repair Request",
                            "My faucet is dripping and it's driving me crazy. Can you fix it?",
                            vec![
                                DialogueChoice {
                                    text: format!(
                                        "I'll send someone right away (${})",
                                        repair_cost
                                    ),
                                    effects: vec![
                                        DialogueEffect::MoneyChange(-repair_cost),
                                        DialogueEffect::HappinessChange {
                                            tenant_id: tenant.id,
                                            amount: 10,
                                        },
                                        DialogueEffect::OpinionChange {
                                            tenant_id: tenant.id,
                                            amount: 5,
                                        },
                                    ],
                                },
                                DialogueChoice {
                                    text: "It's on the list, give me a week".to_string(),
                                    effects: vec![DialogueEffect::HappinessChange {
                                        tenant_id: tenant.id,
                                        amount: -2,
                                    }],
                                },
                            ],
                        )
                    }
                };

                // Avoid duplicates
                if !self
                    .active_dialogues
                    .iter()
                    .any(|d| d.initiator_id == tenant.id)
                {
                    self.add_dialogue(
                        DialogueType::FaceToFaceRequest,
                        tenant.id,
                        None,
                        headline,
                        desc,
                        choices,
                        None,
                    );
                }
            }
        }
    }

    /// Tenant A complaining about Tenant B — sourced from a real hostile
    /// relationship in the tenant network. Generates one conflict at a time.
    fn generate_conflict_mediation(
        &mut self,
        tenants: &[crate::tenant::Tenant],
        network: &crate::consequences::TenantNetwork,
    ) {
        use crate::consequences::RelationshipType;

        let housed = |id: u32| tenants.iter().any(|t| t.id == id);
        let name_of = |id: u32| {
            tenants
                .iter()
                .find(|t| t.id == id)
                .map(|t| t.name.clone())
                .unwrap_or_else(|| "a neighbor".to_string())
        };

        let pair = network.relationships.iter().find(|r| {
            matches!(r.relationship_type, RelationshipType::Hostile)
                && housed(r.tenant_a_id)
                && housed(r.tenant_b_id)
                && !self
                    .active_dialogues
                    .iter()
                    .any(|d| d.initiator_id == r.tenant_a_id)
        });

        let Some(relationship) = pair else {
            return;
        };
        let (a, b) = (relationship.tenant_a_id, relationship.tenant_b_id);
        let a_name = name_of(a);
        let b_name = name_of(b);

        let choices = vec![
            DialogueChoice {
                text: "Mediate between them".to_string(),
                effects: vec![
                    DialogueEffect::RelationshipChange {
                        tenant_a: a,
                        tenant_b: b,
                        change: 15,
                    },
                    DialogueEffect::HappinessChange {
                        tenant_id: a,
                        amount: 3,
                    },
                    DialogueEffect::HappinessChange {
                        tenant_id: b,
                        amount: 3,
                    },
                ],
            },
            DialogueChoice {
                text: format!("Take {}'s side", a_name),
                effects: vec![
                    DialogueEffect::HappinessChange {
                        tenant_id: a,
                        amount: 6,
                    },
                    DialogueEffect::HappinessChange {
                        tenant_id: b,
                        amount: -8,
                    },
                    DialogueEffect::RelationshipChange {
                        tenant_a: a,
                        tenant_b: b,
                        change: -10,
                    },
                ],
            },
            DialogueChoice {
                text: "Stay out of it".to_string(),
                effects: vec![DialogueEffect::HappinessChange {
                    tenant_id: a,
                    amount: -3,
                }],
            },
        ];

        self.add_dialogue(
            DialogueType::ConflictMediation,
            a,
            Some(b),
            "Neighbor Dispute",
            &format!(
                "{} says {} has become impossible to live next to.",
                a_name, b_name
            ),
            choices,
            None,
        );
    }

    /// Rent-change conversations: price-sensitive tenants push back when the
    /// building charges above baseline rent.
    fn generate_rent_negotiations(
        &mut self,
        building: &crate::building::Building,
        tenants: &[crate::tenant::Tenant],
    ) {
        use crate::tenant::TenantArchetype;

        if building.rent_multiplier <= 1.1 {
            return;
        }

        for tenant in tenants {
            let price_sensitive = matches!(
                tenant.archetype,
                TenantArchetype::Elderly | TenantArchetype::Family | TenantArchetype::Student
            );
            if !price_sensitive || tenant.happiness >= 55 {
                continue;
            }
            if self
                .active_dialogues
                .iter()
                .any(|d| d.initiator_id == tenant.id)
            {
                continue;
            }
            if rng::gen_range(0, 100) >= 6 {
                continue;
            }

            let forgiven = 40;
            let choices = vec![
                DialogueChoice {
                    text: format!("Give a one-time ${} break", forgiven),
                    effects: vec![
                        DialogueEffect::MoneyChange(-forgiven),
                        DialogueEffect::HappinessChange {
                            tenant_id: tenant.id,
                            amount: 8,
                        },
                        DialogueEffect::OpinionChange {
                            tenant_id: tenant.id,
                            amount: 5,
                        },
                    ],
                },
                DialogueChoice {
                    text: "Hold firm on the rent".to_string(),
                    effects: vec![
                        DialogueEffect::HappinessChange {
                            tenant_id: tenant.id,
                            amount: -6,
                        },
                        DialogueEffect::OpinionChange {
                            tenant_id: tenant.id,
                            amount: -4,
                        },
                    ],
                },
            ];

            self.add_dialogue(
                DialogueType::RentNegotiation,
                tenant.id,
                None,
                "Rent Is Getting Tight",
                "With the rent where it is, I'm really feeling the squeeze. Any chance of some relief?",
                choices,
                None,
            );
        }
    }

    /// Handle expiring dialogues
    pub fn tick(&mut self, current_month: u32) {
        // Remove expired dialogues
        self.active_dialogues.retain(|d| {
            if let Some(deadline) = d.deadline_month {
                deadline > current_month
            } else {
                true
            }
        });
    }
}

impl Default for DialogueSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialogue_creation() {
        let mut system = DialogueSystem::new();
        let choices = vec![DialogueChoice {
            text: "Yes".to_string(),
            effects: vec![DialogueEffect::MoneyChange(100)],
        }];

        let id = system.add_dialogue(
            DialogueType::FaceToFaceRequest,
            1,
            None,
            "Test",
            "Test Desc",
            choices,
            None,
        );

        assert_eq!(system.pending_dialogues().len(), 1);
        assert_eq!(system.pending_dialogues()[0].id, id);
    }

    #[test]
    fn conflict_mediation_generated_from_hostile_pair() {
        use crate::consequences::TenantNetwork;
        use crate::tenant::{Tenant, TenantArchetype};

        let tenants = vec![
            Tenant::generate(1, TenantArchetype::Professional),
            Tenant::generate(2, TenantArchetype::Artist),
        ];
        let mut network = TenantNetwork::new();
        // A strong negative change with no prior relationship creates a Hostile one.
        network.apply_relationship_change(1, 2, -60);

        let mut system = DialogueSystem::new();
        system.generate_conflict_mediation(&tenants, &network);

        assert!(system
            .active_dialogues
            .iter()
            .any(|d| d.dialogue_type == DialogueType::ConflictMediation && d.target_id == Some(2)));
    }

    #[test]
    fn test_dialogue_resolution() {
        let mut system = DialogueSystem::new();
        let choices = vec![DialogueChoice {
            text: "Yes".to_string(),
            effects: vec![DialogueEffect::MoneyChange(100)],
        }];

        let id = system.add_dialogue(
            DialogueType::FaceToFaceRequest,
            1,
            None,
            "Test",
            "Test Desc",
            choices,
            None,
        );

        let effects = system.resolve_dialogue(id, 0);
        assert!(effects.is_some(), "expected dialogue effects");
        if let Some(effects) = effects {
            assert_eq!(effects.len(), 1);
            assert!(
                matches!(effects[0], DialogueEffect::MoneyChange(100)),
                "expected money change effect"
            );
        }

        assert_eq!(system.pending_dialogues().len(), 0);
    }
}
