use super::{PlayerFunds, Transaction, TransactionType};
use crate::building::Building;
use crate::data::config::TenantRiskConfig;
use crate::tenant::Tenant;
use macroquad_toolkit::rng;

/// Result of rent collection for one tick
#[derive(Clone, Debug)]
pub struct RentCollection {
    pub total_collected: i32,
    pub payments: Vec<RentPayment>,
    pub missed_payments: Vec<MissedPayment>,
}

#[derive(Clone, Debug)]
pub struct RentPayment {
    pub tenant_name: String,
    pub _apartment_unit: String,
    pub amount: i32,
}

#[derive(Clone, Debug)]
pub struct MissedPayment {
    pub tenant_name: String,
    pub _apartment_unit: String,
    pub amount: i32,
    pub _reason: String,
}

/// Collect rent from all tenants
pub fn collect_rent(
    tenants: &[Tenant],
    building: &Building,
    funds: &mut PlayerFunds,
    current_tick: u32,
    risk: &TenantRiskConfig,
) -> RentCollection {
    let mut collection = RentCollection {
        total_collected: 0,
        payments: Vec::new(),
        missed_payments: Vec::new(),
    };

    for tenant in tenants {
        if let Some(apt_id) = tenant.apartment_id {
            if let Some(apartment) = building.get_apartment(apt_id) {
                // Very unhappy tenants might miss payment
                if tenant.happiness < 20 && rng::gen_range(0, 100) < 30 {
                    collection.missed_payments.push(MissedPayment {
                        tenant_name: tenant.name.clone(),
                        _apartment_unit: apartment.unit_number.clone(),
                        amount: apartment.rent_price,
                        _reason: "Tenant too unhappy".to_string(),
                    });
                    continue;
                }

                // Unreliable tenants may skip rent even when otherwise content —
                // this is the cost of accepting an applicant who failed vetting.
                if tenant.rent_reliability < risk.unreliable_threshold
                    && rng::gen_range(0, 100) < risk.skip_rent_chance_percent
                {
                    collection.missed_payments.push(MissedPayment {
                        tenant_name: tenant.name.clone(),
                        _apartment_unit: apartment.unit_number.clone(),
                        amount: apartment.rent_price,
                        _reason: "Unreliable tenant skipped rent".to_string(),
                    });
                    continue;
                }

                let rent = apartment.rent_price;

                funds.add_income(Transaction::income(
                    TransactionType::RentIncome,
                    rent,
                    &format!("Rent from {} (Unit {})", tenant.name, apartment.unit_number),
                    current_tick,
                ));

                collection.payments.push(RentPayment {
                    tenant_name: tenant.name.clone(),
                    _apartment_unit: apartment.unit_number.clone(),
                    amount: rent,
                });

                collection.total_collected += rent;
            }
        }
    }

    collection
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant::{Tenant, TenantArchetype};

    #[test]
    fn unreliable_tenant_skips_rent() {
        let building = Building::new("Test", 1, 1);
        let apt_id = building.apartments[0].id;
        let mut funds = PlayerFunds::new(1000);

        let mut tenant = Tenant::new(1, "Flaky", TenantArchetype::Student);
        tenant.happiness = 80; // avoid the unhappiness skip branch
        tenant.rent_reliability = 10;
        tenant.apartment_id = Some(apt_id);
        let tenants = vec![tenant];

        let risk = TenantRiskConfig {
            unreliable_threshold: 100,
            skip_rent_chance_percent: 100,
            ..TenantRiskConfig::default()
        };

        let collection = collect_rent(&tenants, &building, &mut funds, 1, &risk);
        assert_eq!(collection.total_collected, 0);
        assert_eq!(collection.missed_payments.len(), 1);
    }

    #[test]
    fn reliable_tenant_pays_rent() {
        let building = Building::new("Test", 1, 1);
        let apt_id = building.apartments[0].id;
        let mut funds = PlayerFunds::new(1000);

        let mut tenant = Tenant::new(1, "Solid", TenantArchetype::Professional);
        tenant.happiness = 80;
        tenant.rent_reliability = 95;
        tenant.apartment_id = Some(apt_id);
        let tenants = vec![tenant];

        let collection = collect_rent(
            &tenants,
            &building,
            &mut funds,
            1,
            &TenantRiskConfig::default(),
        );
        assert_eq!(collection.missed_payments.len(), 0);
        assert!(collection.total_collected > 0);
    }
}
