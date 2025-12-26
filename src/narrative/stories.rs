
use serde::{Deserialize, Serialize};
use macroquad::rand::{ChooseRandom, gen_range};
use crate::tenant::TenantArchetype;

/// A story event in a tenant's life
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoryEvent {
    pub month: u32,
    pub description: String,
    pub impact: StoryImpact,
}

/// How a story event affects gameplay
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StoryImpact {
    /// No gameplay effect, just flavor
    None,
    /// Affects happiness
    Happiness(i32),
    /// Affects rent tolerance
    RentTolerance(i32),
    /// Tenant may need to move
    MoveOutRisk(i32),      // 0-100 probability
    /// Tenant requests something
    Request(TenantRequest),
    /// Tenant gets a roommate
    Roommate,
    /// Tenant has life event
    LifeChange(LifeChangeType),
}

/// Types of requests tenants can make
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TenantRequest {
    /// Can I have a pet?
    Pet { pet_type: String },
    /// Can a family member stay temporarily?
    TemporaryGuest { guest_name: String, duration_months: u32 },
    /// Can I run a small business from home?
    HomeBusiness { business_type: String },
    /// Can I modify the apartment?
    Modification { description: String },
    /// Can I sublease part of the unit?
    Sublease,
}

impl TenantRequest {
    /// What happens if the landlord denies
    pub fn denial_effect(&self) -> StoryImpact {
        match self {
            TenantRequest::Pet { .. } => StoryImpact::Happiness(-10),
            TenantRequest::TemporaryGuest { .. } => StoryImpact::Happiness(-5),
            TenantRequest::HomeBusiness { .. } => StoryImpact::Happiness(-8),
            TenantRequest::Modification { .. } => StoryImpact::Happiness(-5),
            TenantRequest::Sublease => StoryImpact::MoveOutRisk(30),
        }
    }
}

/// Types of life changes
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LifeChangeType {
    /// Got a new job
    NewJob { better: bool },
    /// Lost their job
    JobLoss,
    /// Got married/partnered
    Partnered,
    /// Broke up/divorced
    Separated,
    /// Had a baby
    NewBaby,
    /// Child moved out
    ChildLeftHome,
    /// Health issue
    HealthIssue,
    /// Retirement
    Retired,
    /// Starting school
    StartedSchool,
    /// Graduated
    Graduated,
}



/// Complete story/background for a tenant
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TenantStory {
    pub tenant_id: u32,
    
    // Background
    pub job_title: String,
    pub hometown: String,
    pub move_reason: String,
    pub hobbies: Vec<String>,
    pub personality_traits: Vec<String>,
    
    // Family
    pub has_partner: bool,
    pub has_children: bool,
    pub num_children: u32,
    
    // History
    pub story_events: Vec<StoryEvent>,
    
    // Active requests
    pub pending_request: Option<TenantRequest>,
}

impl TenantStory {
    /// Generate a story for a new tenant
    pub fn generate(tenant_id: u32, archetype: &TenantArchetype) -> Self {
        let generator = BackgroundGenerator::default();
        generator.generate(tenant_id, archetype)
    }

    /// Add a story event
    pub fn add_event(&mut self, month: u32, description: &str, impact: StoryImpact) {
        self.story_events.push(StoryEvent {
            month,
            description: description.to_string(),
            impact,
        });
    }

    /// Make a random request based on archetype
    pub fn make_request(&mut self, archetype: &TenantArchetype) {
        if self.pending_request.is_some() {
            return;
        }

        let request = match archetype {
            TenantArchetype::Student => {
                if gen_range(0, 100) < 30 {
                    Some(TenantRequest::Pet {
                        pet_type: vec!["cat", "small dog", "hamster"].choose().unwrap().to_string(),
                    })
                } else if gen_range(0, 100) < 20 {
                    Some(TenantRequest::Sublease)
                } else {
                    None
                }
            }
            TenantArchetype::Professional => {
                if gen_range(0, 100) < 20 {
                    Some(TenantRequest::HomeBusiness {
                        business_type: vec!["consulting", "freelance work", "tutoring"].choose().unwrap().to_string(),
                    })
                } else {
                    None
                }
            }
            TenantArchetype::Artist => {
                if gen_range(0, 100) < 40 {
                    Some(TenantRequest::HomeBusiness {
                        business_type: vec!["art studio", "music lessons", "craft workshop"].choose().unwrap().to_string(),
                    })
                } else if gen_range(0, 100) < 20 {
                    Some(TenantRequest::Modification {
                        description: vec![
                            "install better lighting",
                            "add a small loft for storage",
                            "paint the walls a different color"
                        ].choose().unwrap().to_string(),
                    })
                } else {
                    None
                }
            }
            TenantArchetype::Family => {
                if gen_range(0, 100) < 40 {
                    Some(TenantRequest::Pet {
                        pet_type: vec!["dog", "cat", "goldfish"].choose().unwrap().to_string(),
                    })
                } else if gen_range(0, 100) < 30 {
                    Some(TenantRequest::TemporaryGuest {
                        guest_name: format!("{} (relative)", 
                            vec!["Grandma", "Grandpa", "Aunt", "Uncle", "Cousin"].choose().unwrap()),
                        duration_months: gen_range(1, 4),
                    })
                } else {
                    None
                }
            }
            TenantArchetype::Elderly => {
                if gen_range(0, 100) < 30 {
                    Some(TenantRequest::Pet {
                        pet_type: vec!["cat", "small dog", "bird"].choose().unwrap().to_string(),
                    })
                } else if gen_range(0, 100) < 20 {
                    Some(TenantRequest::TemporaryGuest {
                        guest_name: format!("{} (caregiver)", 
                            vec!["my niece", "my nephew", "a nurse", "my friend"].choose().unwrap()),
                        duration_months: gen_range(1, 3),
                    })
                } else {
                    None
                }
            }
        };

        self.pending_request = request;
    }


}

/// Generates tenant backgrounds
pub struct BackgroundGenerator {
    job_titles: std::collections::HashMap<TenantArchetype, Vec<&'static str>>,
    hometowns: Vec<&'static str>,
    move_reasons: std::collections::HashMap<TenantArchetype, Vec<&'static str>>,
    hobbies: std::collections::HashMap<TenantArchetype, Vec<&'static str>>,
    traits: Vec<&'static str>,
}

impl BackgroundGenerator {
    pub fn generate(&self, tenant_id: u32, archetype: &TenantArchetype) -> TenantStory {
        let job_title = self.job_titles.get(archetype)
            .and_then(|jobs| jobs.choose().copied())
            .unwrap_or("Worker")
            .to_string();

        let hometown = self.hometowns.choose().copied().unwrap_or("Unknown").to_string();

        let move_reason = self.move_reasons.get(archetype)
            .and_then(|reasons| reasons.choose().copied())
            .unwrap_or("Needed a change")
            .to_string();

        let hobby_pool = self.hobbies.get(archetype).cloned().unwrap_or_default();
        let num_hobbies = gen_range(1, 3);
        let hobbies: Vec<String> = hobby_pool.choose_multiple(num_hobbies)
            .map(|s| s.to_string())
            .collect();

        let num_traits = gen_range(1, 3);
        let personality_traits: Vec<String> = self.traits.iter()
            .cloned()
            .collect::<Vec<_>>()
            .choose_multiple(num_traits)
            .map(|s| s.to_string())
            .collect();

        let (has_partner, has_children, num_children) = match archetype {
            TenantArchetype::Student => (false, false, 0),
            TenantArchetype::Professional => (gen_range(0, 100) < 30, false, 0),
            TenantArchetype::Artist => (gen_range(0, 100) < 20, false, 0),
            TenantArchetype::Family => (true, true, gen_range(1, 4)),
            TenantArchetype::Elderly => (gen_range(0, 100) < 50, gen_range(0, 100) < 70, 0),
        };

        TenantStory {
            tenant_id,
            job_title,
            hometown,
            move_reason,
            hobbies,
            personality_traits,
            has_partner,
            has_children,
            num_children,
            story_events: Vec::new(),
            pending_request: None,
        }
    }
}

impl Default for BackgroundGenerator {
    fn default() -> Self {
        use std::collections::HashMap;

        let mut job_titles = HashMap::new();
        job_titles.insert(TenantArchetype::Student, vec![
            "University Student", "Graduate Student", "Community College Student",
            "Trade School Student", "Exchange Student", "Medical Student",
        ]);
        job_titles.insert(TenantArchetype::Professional, vec![
            "Software Developer", "Accountant", "Marketing Manager", "Lawyer",
            "Project Manager", "Financial Analyst", "Consultant", "Doctor",
            "Engineer", "Architect",
        ]);
        job_titles.insert(TenantArchetype::Artist, vec![
            "Painter", "Musician", "Writer", "Photographer", "Graphic Designer",
            "Sculptor", "Filmmaker", "Dancer", "Potter", "Illustrator",
        ]);
        job_titles.insert(TenantArchetype::Family, vec![
            "Teacher", "Nurse", "Small Business Owner", "Sales Representative",
            "Office Manager", "Electrician", "Chef", "Social Worker",
        ]);
        job_titles.insert(TenantArchetype::Elderly, vec![
            "Retired Teacher", "Retired Accountant", "Retired Nurse",
            "Retired Factory Worker", "Retired Business Owner", "Widower",
        ]);

        let hometowns = vec![
            "the suburbs", "a small town", "across the country", "overseas",
            "downtown", "the countryside", "another city", "up north",
            "the coast", "the midwest",
        ];

        let mut move_reasons = HashMap::new();
        move_reasons.insert(TenantArchetype::Student, vec![
            "Started at the local university.", "Needed to be closer to campus.",
            "Looking for affordable housing near school.", "Moving for an internship.",
        ]);
        move_reasons.insert(TenantArchetype::Professional, vec![
            "Got a new job in the area.", "Wanted a shorter commute.",
            "Looking for a quieter neighborhood.", "Relocated for work.",
        ]);
        move_reasons.insert(TenantArchetype::Artist, vec![
            "Looking for an inspiring space.", "Needed a studio with good light.",
            "Drawn to the creative community here.", "Escaping the high rents elsewhere.",
        ]);
        move_reasons.insert(TenantArchetype::Family, vec![
            "Needed more space for the kids.", "Moving for the school district.",
            "Wanted a safer neighborhood.", "Growing family needs.",
        ]);
        move_reasons.insert(TenantArchetype::Elderly, vec![
            "Downsizing after retirement.", "Wanted to be closer to family.",
            "Looking for a quieter place.", "Needed a ground floor unit.",
        ]);

        let mut hobbies = HashMap::new();
        hobbies.insert(TenantArchetype::Student, vec![
            "gaming", "studying", "partying", "jogging", "reading",
            "cooking on a budget", "streaming", "yoga",
        ]);
        hobbies.insert(TenantArchetype::Professional, vec![
            "wine tasting", "golf", "reading", "fitness", "travel",
            "cooking", "podcasts", "networking events",
        ]);
        hobbies.insert(TenantArchetype::Artist, vec![
            "painting", "music", "writing", "photography", "sculpting",
            "gallery hopping", "poetry readings", "experimental cooking",
        ]);
        hobbies.insert(TenantArchetype::Family, vec![
            "family outings", "cooking", "gardening", "board games",
            "soccer practice", "movie nights", "camping",
        ]);
        hobbies.insert(TenantArchetype::Elderly, vec![
            "gardening", "crossword puzzles", "watching TV", "knitting",
            "reading", "bird watching", "walking", "bingo",
        ]);

        let traits = vec![
            "quiet", "friendly", "private", "social", "neat", "messy",
            "punctual", "easygoing", "strict", "flexible", "chatty", "reserved",
        ];

        Self {
            job_titles,
            hometowns,
            move_reasons,
            hobbies,
            traits,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_generation() {
        let story = TenantStory::generate(0, &TenantArchetype::Student);
        assert!(!story.job_title.is_empty());
        assert!(!story.hometown.is_empty());
    }

    #[test]
    fn test_request_effects() {
        let request = TenantRequest::Pet { pet_type: "cat".to_string() };
        let approval = request.approval_effect();
        matches!(approval, StoryImpact::Happiness(h) if h > 0);
    }
}
