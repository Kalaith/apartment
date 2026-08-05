//! Scripted player policies used by the representative balance sweep.

/// A scripted player policy. Every field is a lever the harness pulls each month.
#[derive(Clone, Copy)]
pub(super) struct Strategy {
    pub(super) name: &'static str,
    pub(super) vet_applicants: bool,
    pub(super) repair_threshold: i32,
    pub(super) upgrade_designs: bool,
    pub(super) hire_staff: bool,
    pub(super) tenant_services: bool,
    pub(super) special: SpecialPolicy,
    pub(super) cash_reserve: i32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SpecialPolicy {
    None,
    AggressiveRent,
    CondoSale,
    PortfolioExpansion,
}

pub(super) fn strategies() -> Vec<Strategy> {
    vec![
        Strategy {
            name: "Greedy (accept-all, minimum compliance)",
            vet_applicants: false,
            repair_threshold: 48,
            upgrade_designs: false,
            hire_staff: false,
            tenant_services: false,
            special: SpecialPolicy::None,
            cash_reserve: 500,
        },
        Strategy {
            name: "Investor (vet, improve, staff, services)",
            vet_applicants: true,
            repair_threshold: 75,
            upgrade_designs: true,
            hire_staff: true,
            tenant_services: true,
            special: SpecialPolicy::None,
            cash_reserve: 3_000,
        },
        Strategy {
            name: "Neglect (accept-all, no upkeep)",
            vet_applicants: false,
            repair_threshold: 0,
            upgrade_designs: false,
            hire_staff: false,
            tenant_services: false,
            special: SpecialPolicy::None,
            cash_reserve: 0,
        },
        Strategy {
            name: "Rent maximizer (+15% every 6 months)",
            vet_applicants: false,
            repair_threshold: 55,
            upgrade_designs: false,
            hire_staff: false,
            tenant_services: false,
            special: SpecialPolicy::AggressiveRent,
            cash_reserve: 500,
        },
        Strategy {
            name: "Condo liquidity (one sale at month 12)",
            vet_applicants: false,
            repair_threshold: 48,
            upgrade_designs: false,
            hire_staff: false,
            tenant_services: false,
            special: SpecialPolicy::CondoSale,
            cash_reserve: 1_000,
        },
        Strategy {
            name: "Portfolio saver (cash expansion)",
            vet_applicants: true,
            repair_threshold: 55,
            upgrade_designs: false,
            hire_staff: false,
            tenant_services: true,
            special: SpecialPolicy::PortfolioExpansion,
            cash_reserve: 3_000,
        },
    ]
}
