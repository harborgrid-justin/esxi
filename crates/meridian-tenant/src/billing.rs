//! Tenant billing integration hooks and usage tracking.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{TenantError, TenantResult};
use crate::tenant::TenantTier;

/// Billing cycle types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BillingCycle {
    Monthly,
    Quarterly,
    Yearly,
    Custom,
}

/// Billing status for a tenant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BillingStatus {
    Active,
    Trialing,
    PastDue,
    Suspended,
    Canceled,
}

/// Payment method types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    CreditCard,
    BankTransfer,
    PayPal,
    Invoice,
    Other,
}

/// Billing account for a tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingAccount {
    pub tenant_id: Uuid,
    pub status: BillingStatus,
    pub tier: TenantTier,
    pub billing_cycle: BillingCycle,
    pub payment_method: Option<PaymentMethod>,
    pub billing_email: String,
    pub billing_address: Option<BillingAddress>,
    pub trial_ends_at: Option<DateTime<Utc>>,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub next_billing_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingAddress {
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: String,
    pub country: String,
}

/// Billable usage event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub event_type: String,
    pub quantity: Decimal,
    pub unit: String,
    pub unit_price: Option<Decimal>,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

impl UsageEvent {
    pub fn new(
        tenant_id: Uuid,
        event_type: impl Into<String>,
        quantity: Decimal,
        unit: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            event_type: event_type.into(),
            quantity,
            unit: unit.into(),
            unit_price: None,
            timestamp: Utc::now(),
            metadata: serde_json::json!({}),
        }
    }

    pub fn with_price(mut self, price: Decimal) -> Self {
        self.unit_price = Some(price);
        self
    }

    pub fn calculate_cost(&self) -> Option<Decimal> {
        self.unit_price.map(|price| price * self.quantity)
    }
}

/// Invoice for a billing period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub invoice_number: String,
    pub status: InvoiceStatus,
    pub subtotal: Decimal,
    pub tax: Decimal,
    pub total: Decimal,
    pub currency: String,
    pub line_items: Vec<InvoiceLineItem>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub issued_at: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InvoiceStatus {
    Draft,
    Open,
    Paid,
    Void,
    Uncollectible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLineItem {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub amount: Decimal,
    pub metadata: serde_json::Value,
}

/// Subscription plan details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPlan {
    pub id: Uuid,
    pub name: String,
    pub tier: TenantTier,
    pub price: Decimal,
    pub currency: String,
    pub billing_cycle: BillingCycle,
    pub features: Vec<String>,
    pub quotas: HashMap<String, u64>,
    pub trial_days: Option<u32>,
}

impl SubscriptionPlan {
    pub fn free_plan() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Free".to_string(),
            tier: TenantTier::Free,
            price: Decimal::ZERO,
            currency: "USD".to_string(),
            billing_cycle: BillingCycle::Monthly,
            features: vec!["basic_gis".to_string(), "5_layers".to_string()],
            quotas: HashMap::from([
                ("api_requests".to_string(), 1000),
                ("storage_mb".to_string(), 100),
            ]),
            trial_days: None,
        }
    }

    pub fn professional_plan() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Professional".to_string(),
            tier: TenantTier::Professional,
            price: Decimal::new(99, 0),
            currency: "USD".to_string(),
            billing_cycle: BillingCycle::Monthly,
            features: vec![
                "advanced_gis".to_string(),
                "100_layers".to_string(),
                "custom_branding".to_string(),
                "api_access".to_string(),
            ],
            quotas: HashMap::from([
                ("api_requests".to_string(), 100_000),
                ("storage_mb".to_string(), 10_000),
            ]),
            trial_days: Some(14),
        }
    }
}

/// Billing manager for handling tenant billing operations.
pub struct BillingManager {
    accounts: HashMap<Uuid, BillingAccount>,
    usage_events: Vec<UsageEvent>,
    invoices: HashMap<Uuid, Invoice>,
    plans: HashMap<Uuid, SubscriptionPlan>,
}

impl BillingManager {
    pub fn new() -> Self {
        let mut plans = HashMap::new();

        let free_plan = SubscriptionPlan::free_plan();
        let pro_plan = SubscriptionPlan::professional_plan();

        plans.insert(free_plan.id, free_plan);
        plans.insert(pro_plan.id, pro_plan);

        Self {
            accounts: HashMap::new(),
            usage_events: Vec::new(),
            invoices: HashMap::new(),
            plans,
        }
    }

    /// Creates a billing account for a tenant.
    pub fn create_account(
        &mut self,
        tenant_id: Uuid,
        tier: TenantTier,
        billing_email: impl Into<String>,
    ) -> TenantResult<BillingAccount> {
        let now = Utc::now();
        let account = BillingAccount {
            tenant_id,
            status: BillingStatus::Active,
            tier,
            billing_cycle: BillingCycle::Monthly,
            payment_method: None,
            billing_email: billing_email.into(),
            billing_address: None,
            trial_ends_at: None,
            current_period_start: now,
            current_period_end: now + chrono::Duration::days(30),
            next_billing_date: Some(now + chrono::Duration::days(30)),
            created_at: now,
            updated_at: now,
            metadata: serde_json::json!({}),
        };

        self.accounts.insert(tenant_id, account.clone());
        Ok(account)
    }

    /// Records a usage event.
    pub fn record_usage(&mut self, event: UsageEvent) -> TenantResult<()> {
        tracing::info!(
            "Recording usage event: {} for tenant {}",
            event.event_type,
            event.tenant_id
        );

        self.usage_events.push(event);
        Ok(())
    }

    /// Gets usage summary for a tenant in a date range.
    pub fn get_usage_summary(
        &self,
        tenant_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> UsageSummary {
        let events: Vec<&UsageEvent> = self
            .usage_events
            .iter()
            .filter(|e| {
                e.tenant_id == tenant_id && e.timestamp >= start && e.timestamp <= end
            })
            .collect();

        let mut by_type = HashMap::new();
        let mut total_cost = Decimal::ZERO;

        for event in &events {
            *by_type.entry(event.event_type.clone()).or_insert(Decimal::ZERO) += event.quantity;

            if let Some(cost) = event.calculate_cost() {
                total_cost += cost;
            }
        }

        UsageSummary {
            tenant_id,
            period_start: start,
            period_end: end,
            total_events: events.len(),
            usage_by_type: by_type,
            estimated_cost: total_cost,
        }
    }

    /// Generates an invoice for a tenant.
    pub fn generate_invoice(
        &mut self,
        tenant_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> TenantResult<Invoice> {
        let account = self.accounts.get(&tenant_id).ok_or_else(|| {
            TenantError::BillingError(format!("No billing account for tenant: {}", tenant_id))
        })?;

        let summary = self.get_usage_summary(tenant_id, period_start, period_end);
        let mut line_items = Vec::new();

        // Add subscription line item
        if let Some(plan) = self.plans.values().find(|p| p.tier == account.tier) {
            line_items.push(InvoiceLineItem {
                description: format!("{} Plan - Subscription", plan.name),
                quantity: Decimal::ONE,
                unit_price: plan.price,
                amount: plan.price,
                metadata: serde_json::json!({}),
            });
        }

        // Add usage-based line items
        for (event_type, quantity) in &summary.usage_by_type {
            line_items.push(InvoiceLineItem {
                description: format!("Usage: {}", event_type),
                quantity: *quantity,
                unit_price: Decimal::ZERO, // Would lookup pricing
                amount: Decimal::ZERO,
                metadata: serde_json::json!({}),
            });
        }

        let subtotal: Decimal = line_items.iter().map(|item| item.amount).sum();
        let tax = subtotal * Decimal::new(10, 2); // 10% tax
        let total = subtotal + tax;

        let invoice = Invoice {
            id: Uuid::new_v4(),
            tenant_id,
            invoice_number: format!("INV-{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
            status: InvoiceStatus::Open,
            subtotal,
            tax,
            total,
            currency: "USD".to_string(),
            line_items,
            period_start,
            period_end,
            issued_at: Utc::now(),
            due_date: Utc::now() + chrono::Duration::days(15),
            paid_at: None,
            metadata: serde_json::json!({}),
        };

        self.invoices.insert(invoice.id, invoice.clone());
        Ok(invoice)
    }

    /// Marks an invoice as paid.
    pub fn mark_invoice_paid(&mut self, invoice_id: Uuid) -> TenantResult<()> {
        let invoice = self.invoices.get_mut(&invoice_id).ok_or_else(|| {
            TenantError::BillingError("Invoice not found".to_string())
        })?;

        invoice.status = InvoiceStatus::Paid;
        invoice.paid_at = Some(Utc::now());

        Ok(())
    }

    /// Suspends billing for a tenant.
    pub fn suspend_account(&mut self, tenant_id: Uuid) -> TenantResult<()> {
        let account = self.accounts.get_mut(&tenant_id).ok_or_else(|| {
            TenantError::BillingError("Billing account not found".to_string())
        })?;

        account.status = BillingStatus::Suspended;
        account.updated_at = Utc::now();

        Ok(())
    }
}

impl Default for BillingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Usage summary for a billing period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub tenant_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_events: usize,
    pub usage_by_type: HashMap<String, Decimal>,
    pub estimated_cost: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_event() {
        let event = UsageEvent::new(
            Uuid::new_v4(),
            "api_request",
            Decimal::new(100, 0),
            "requests",
        )
        .with_price(Decimal::new(1, 2)); // $0.01

        let cost = event.calculate_cost();
        assert_eq!(cost, Some(Decimal::new(100, 2))); // $1.00
    }

    #[test]
    fn test_billing_manager() {
        let mut manager = BillingManager::new();
        let tenant_id = Uuid::new_v4();

        let account = manager
            .create_account(tenant_id, TenantTier::Professional, "billing@example.com")
            .unwrap();

        assert_eq!(account.tenant_id, tenant_id);
        assert_eq!(account.status, BillingStatus::Active);
    }

    #[test]
    fn test_subscription_plans() {
        let free = SubscriptionPlan::free_plan();
        assert_eq!(free.price, Decimal::ZERO);

        let pro = SubscriptionPlan::professional_plan();
        assert!(pro.price > Decimal::ZERO);
        assert_eq!(pro.trial_days, Some(14));
    }
}
