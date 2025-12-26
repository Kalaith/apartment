# Phase 4B: Tenant Lifecycle & Acquisition

## Goal
Give players agency over who lives in their buildings and how they find them, rather than relying on pure randomness.

## Features

### 1. Active Marketing
Players choose how to fill vacancies.
- **Marketing Options**:
  - **Social Media ($50)**: High volume, low vetting, attracts Students/Artists.
  - **Local Newspaper ($150)**: Moderate volume, attracts Elderly/Families.
  - **Premium Agency ($500)**: Low volume, high "Match Score" boost, attracts Professionals.
- **Open House**: An action that boosts application rates for a specific building for 1 month.

### 2. The Vetting UI
When an application arrives, players can perform "checks" for a fee.
- **Credit Check ($25)**: Reveals "Rent Reliability" (hidden stat).
- **Previous Landlord Call ($10)**: Reveals "Noise/Behavior" history.
- **Security Deposit Negotiation**: Ask for 1, 2, or 3 months' rent upfront. Higher deposits scare away low-budget tenants but protect against defaults.

### 3. Move-In/Move-Out Depth
- **Cleaning Fee**: Charge tenants or pay for cleaning between leases.
- **Lease Terms**: 6-month vs 12-month leases. Short leases = higher rent but higher turnover risk.

## Technical Tasks
- [ ] Expand `TenantApplication` with "Hidden Traits."
- [ ] Implementation of `MarketingCampaign` state in `Building`.
- [ ] UI for Vetting (showing check results).
- [ ] Lease negotiation logic in `src/tenant/matching.rs`.
