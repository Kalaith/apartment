use serde::{Deserialize, Serialize};

/// Types of mail items
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MailType {
    /// Letter from a tenant
    TenantLetter { tenant_id: u32 },
    /// City notice or fine
    CityNotice,
    /// Financial statement
    Financial,
    /// Advertisement from vendors
    Advertisement,
    /// Newspaper clipping about the neighborhood
    News,
    /// Invitation or personal correspondence
    Personal,
    /// Official document
    Official,
}

impl MailType {
    pub fn icon(&self) -> &'static str {
        match self {
            MailType::TenantLetter { .. } => "📬",
            MailType::CityNotice => "🏛️",
            MailType::Financial => "💰",
            MailType::Advertisement => "📰",
            MailType::News => "📰",
            MailType::Personal => "💌",
            MailType::Official => "📋",
        }
    }

    pub fn priority(&self) -> i32 {
        match self {
            MailType::CityNotice => 3,
            MailType::Official => 3,
            MailType::TenantLetter { .. } => 2,
            MailType::Financial => 2,
            MailType::Personal => 1,
            MailType::News => 0,
            MailType::Advertisement => 0,
        }
    }
}

/// A mail item in the player's mailbox
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MailItem {
    pub id: u32,
    pub mail_type: MailType,
    pub month_received: u32,
    pub sender: String,
    pub subject: String,
    pub body: String,
    pub read: bool,
    /// Associated action if any
    pub action: Option<MailAction>,
    /// If true, must be dealt with
    pub requires_attention: bool,
}

/// Actions that can be taken from mail
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MailAction {
    /// Pay a fine or fee
    PayFine { amount: i32, deadline_month: u32 },
    /// Respond to a tenant request
    RespondToTenant { tenant_id: u32, request_type: String },
    /// Schedule an inspection
    ScheduleInspection { building_id: u32 },
    /// Accept or reject an offer
    Offer { amount: i32, expires_month: u32 },
    /// Just acknowledge
    Acknowledge,
}

impl MailItem {
    /// Create a tenant letter
    pub fn tenant_letter(
        id: u32,
        tenant_id: u32,
        tenant_name: &str,
        month: u32,
        subject: &str,
        body: &str,
    ) -> Self {
        Self {
            id,
            mail_type: MailType::TenantLetter { tenant_id },
            month_received: month,
            sender: tenant_name.to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
            read: false,
            action: None,
            requires_attention: false,
        }
    }

    /// Create a city notice
    pub fn city_notice(
        id: u32,
        month: u32,
        subject: &str,
        body: &str,
        action: Option<MailAction>,
    ) -> Self {
        let requires_attention = action.is_some();
        Self {
            id,
            mail_type: MailType::CityNotice,
            month_received: month,
            sender: "City of Metropolis".to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
            read: false,
            action,
            requires_attention,
        }
    }

    /// Create a financial statement
    pub fn financial_statement(
        id: u32,
        month: u32,
        income: i32,
        expenses: i32,
        net: i32,
    ) -> Self {
        let body = format!(
            "Monthly Financial Summary:\n\n\
             Total Income: ${}\n\
             Total Expenses: ${}\n\
             ─────────────────\n\
             Net: ${}\n\n\
             Keep up the good work!",
            income, expenses, net
        );

        Self {
            id,
            mail_type: MailType::Financial,
            month_received: month,
            sender: "Property Management Office".to_string(),
            subject: format!("Monthly Statement - Month {}", month),
            body,
            read: false,
            action: None,
            requires_attention: net < 0,
        }
    }

    /// Create a news clipping
    pub fn news_clipping(id: u32, month: u32, headline: &str, article: &str) -> Self {
        Self {
            id,
            mail_type: MailType::News,
            month_received: month,
            sender: "The Daily Herald".to_string(),
            subject: headline.to_string(),
            body: article.to_string(),
            read: false,
            action: None,
            requires_attention: false,
        }
    }

    /// Mark as read
    pub fn mark_read(&mut self) {
        self.read = true;
    }

    /// Get age in months
    pub fn age(&self, current_month: u32) -> u32 {
        current_month.saturating_sub(self.month_received)
    }
}

/// Player's mailbox containing all mail
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mailbox {
    pub items: Vec<MailItem>,
    next_id: u32,
    /// Unread count (cached for quick access)
    unread_count: usize,
}

impl Mailbox {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            next_id: 0,
            unread_count: 0,
        }
    }

    /// Add new mail
    pub fn receive(&mut self, mut item: MailItem) {
        item.id = self.next_id;
        self.next_id += 1;
        self.unread_count += 1;
        self.items.push(item);
    }

    /// Get unread count
    pub fn unread_count(&self) -> usize {
        self.unread_count
    }

    /// Get mail requiring attention
    pub fn needs_attention(&self) -> Vec<&MailItem> {
        self.items.iter()
            .filter(|m| m.requires_attention && !m.read)
            .collect()
    }

    /// Get all unread mail sorted by priority
    pub fn unread_mail(&self) -> Vec<&MailItem> {
        let mut unread: Vec<_> = self.items.iter().filter(|m| !m.read).collect();
        unread.sort_by(|a, b| b.mail_type.priority().cmp(&a.mail_type.priority()));
        unread
    }

    /// Mark item as read
    pub fn mark_read(&mut self, id: u32) {
        if let Some(item) = self.items.iter_mut().find(|m| m.id == id) {
            if !item.read {
                item.read = true;
                self.unread_count = self.unread_count.saturating_sub(1);
            }
        }
    }

    /// Get a specific mail item
    pub fn get(&self, id: u32) -> Option<&MailItem> {
        self.items.iter().find(|m| m.id == id)
    }

    /// Get mutable reference to a mail item
    pub fn get_mut(&mut self, id: u32) -> Option<&mut MailItem> {
        self.items.iter_mut().find(|m| m.id == id)
    }

    /// Delete old read mail (cleanup)
    pub fn cleanup(&mut self, current_month: u32, max_age_months: u32) {
        self.items.retain(|m| {
            // Keep unread mail, mail needing attention, or recent mail
            !m.read || m.requires_attention || m.age(current_month) < max_age_months
        });
    }

    /// Generate periodic mail
    pub fn generate_mail(
        &mut self,
        month: u32,
        income: i32,
        expenses: i32,
        tenants: &[crate::tenant::Tenant],
        buildings: &[crate::building::Building],
    ) {
        // Monthly financial statement
        if month > 0 {
            let statement = MailItem::financial_statement(
                0, month, income, expenses, income - expenses
            );
            self.receive(statement);
        }

        // Random tenant letters
        for tenant in tenants {
            if macroquad::rand::gen_range(0, 100) < 10 {
                let letter = self.generate_tenant_letter(month, tenant, buildings);
                if let Some(l) = letter {
                    self.receive(l);
                }
            }
        }

        // Occasional news
        if macroquad::rand::gen_range(0, 100) < 15 {
            let headlines = vec![
                ("Housing Market Update", "Rental prices continue to rise across the city as demand increases."),
                ("Local Business Spotlight", "New shops opening in several neighborhoods."),
                ("Community News", "Residents organize neighborhood watch program."),
                ("Weather Alert", "Unusual weather pattern expected this season."),
            ];
            
            use macroquad::rand::ChooseRandom;
            if let Some((headline, article)) = headlines.choose() {
                self.receive(MailItem::news_clipping(0, month, headline, article));
            }
        }
    }

    fn generate_tenant_letter(
        &self,
        month: u32,
        tenant: &crate::tenant::Tenant,
        buildings: &[crate::building::Building],
    ) -> Option<MailItem> {
        use macroquad::rand::ChooseRandom;

        // Find tenant's apartment
        let apt = tenant.apartment_id.and_then(|apt_id| {
            buildings.iter()
                .flat_map(|b| &b.apartments)
                .find(|a| a.id == apt_id)
        })?;

        let templates: Vec<(&str, String)> = match () {
            _ if apt.condition < 40 => {
                vec![
                    ("Maintenance Request", format!(
                        "Dear Landlord,\n\n\
                        I've noticed the apartment is getting quite worn down. \
                        Would it be possible to get some repairs done soon?\n\n\
                        Thank you,\n{}", tenant.name
                    )),
                ]
            }
            _ if tenant.happiness > 80 => {
                vec![
                    ("Thank You Note", format!(
                        "Dear Landlord,\n\n\
                        I just wanted to say I really appreciate how well you \
                        maintain the building. It's a pleasure living here!\n\n\
                        Best,\n{}", tenant.name
                    )),
                ]
            }
            _ if tenant.happiness < 40 => {
                vec![
                    ("Concerns", format!(
                        "Dear Landlord,\n\n\
                        I have some concerns about my unit that I'd like to discuss. \
                        Please let me know when you're available to talk.\n\n\
                        Regards,\n{}", tenant.name
                    )),
                ]
            }
            _ => {
                vec![
                    ("Quick Note", format!(
                        "Hi there,\n\n\
                        Just a friendly check-in. Everything's going well!\n\n\
                        Cheers,\n{}", tenant.name
                    )),
                ]
            }
        };

        templates.choose().map(|(subject, body)| {
            MailItem::tenant_letter(0, tenant.id, &tenant.name, month, subject, body)
        })
    }

    /// Recent mail
    pub fn recent(&self, count: usize) -> Vec<&MailItem> {
        self.items.iter().rev().take(count).collect()
    }
}

impl Default for Mailbox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mailbox_basics() {
        let mut mailbox = Mailbox::new();
        mailbox.receive(MailItem::news_clipping(0, 1, "Test", "Body"));
        
        assert_eq!(mailbox.unread_count(), 1);
        
        let id = mailbox.items[0].id;
        mailbox.mark_read(id);
        
        assert_eq!(mailbox.unread_count(), 0);
    }

    #[test]
    fn test_mail_priority() {
        assert!(MailType::CityNotice.priority() > MailType::Advertisement.priority());
    }
}
