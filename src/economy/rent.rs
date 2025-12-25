use macroquad::rand::gen_range;
use crate::building::Building;
use crate::tenant::Tenant;
use super::{Transaction, TransactionType, PlayerFunds};

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
    pub apartment_unit: String,
    pub amount: i32,
}

#[derive(Clone, Debug)]
pub struct MissedPayment {
    pub tenant_name: String,
    pub apartment_unit: String,
    pub amount: i32,
    pub reason: String,
}

/// Collect rent from all tenants
pub fn collect_rent(
    tenants: &[Tenant],
    building: &Building,
    funds: &mut PlayerFunds,
    current_tick: u32,
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
                if tenant.happiness < 20 && gen_range(0, 100) < 30 {
                    collection.missed_payments.push(MissedPayment {
                        tenant_name: tenant.name.clone(),
                        apartment_unit: apartment.unit_number.clone(),
                        amount: apartment.rent_price,
                        reason: "Tenant too unhappy".to_string(),
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
                    apartment_unit: apartment.unit_number.clone(),
                    amount: rent,
                });
                
                collection.total_collected += rent;
            }
        }
    }
    
    collection
}

/// Calculate expected monthly rent (for projections)
pub fn calculate_expected_rent(tenants: &[Tenant], building: &Building) -> i32 {
    tenants.iter()
        .filter_map(|t| t.apartment_id)
        .filter_map(|apt_id| building.get_apartment(apt_id))
        .map(|apt| apt.rent_price)
        .sum()
}

/// Calculate potential rent if fully occupied
pub fn calculate_max_potential_rent(building: &Building) -> i32 {
    building.apartments.iter().map(|apt| apt.rent_price).sum()
}

/// Calculate occupancy rate as percentage
pub fn calculate_occupancy_rate(building: &Building) -> f32 {
    let total = building.apartments.len() as f32;
    if total == 0.0 {
        return 0.0;
    }
    let occupied = building.occupancy_count() as f32;
    (occupied / total) * 100.0
}
