
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
                        _apartment_unit: apartment.unit_number.clone(),
                        amount: apartment.rent_price,
                        _reason: "Tenant too unhappy".to_string(),
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


