use crate::components::{access_control, merchant};
use crate::errors::ContractError;
use crate::events;
use crate::types::{DataKey, Invoice, InvoiceFilter, InvoiceStatus, Role};
use soroban_sdk::{panic_with_error, token, Address, Env, String, Vec};

pub fn create_invoice(
    env: &Env,
    merchant_address: &Address,
    description: &String,
    amount: i128,
    token: &Address,
) -> u64 {
    merchant_address.require_auth();

    if amount <= 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    if !merchant::is_merchant(env, merchant_address) {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    let merchant_id: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::MerchantId(merchant_address.clone()))
        .unwrap();

    let invoice_count: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::InvoiceCount)
        .unwrap_or(0);

    let new_invoice_id = invoice_count + 1;

    let invoice = Invoice {
        id: new_invoice_id,
        description: description.clone(),
        amount,
        token: token.clone(),
        status: InvoiceStatus::Pending,
        merchant_id,
        payer: None,
        date_created: env.ledger().timestamp(),
        date_paid: None,
    };

    env.storage()
        .persistent()
        .set(&DataKey::Invoice(new_invoice_id), &invoice);
    env.storage()
        .persistent()
        .set(&DataKey::InvoiceCount, &new_invoice_id);

    events::publish_invoice_created_event(
        env,
        new_invoice_id,
        merchant_address.clone(),
        amount,
        token.clone(),
    );

    new_invoice_id
}

pub fn get_invoice(env: &Env, invoice_id: u64) -> Invoice {
    env.storage()
        .persistent()
        .get(&DataKey::Invoice(invoice_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::InvoiceNotFound))
}

pub fn get_invoices(env: &Env, filter: InvoiceFilter) -> Vec<Invoice> {
    let invoice_count: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::InvoiceCount)
        .unwrap_or(0);

    let mut invoices: Vec<Invoice> = Vec::new(env);

    for i in 1..=invoice_count {
        if let Some(invoice) = env
            .storage()
            .persistent()
            .get::<_, Invoice>(&DataKey::Invoice(i))
        {
            let mut matches = true;

            if let Some(status) = filter.status {
                if invoice.status as u32 != status {
                    matches = false;
                }
            }

            if let Some(merchant) = &filter.merchant {
                if let Some(merchant_id) = env
                    .storage()
                    .persistent()
                    .get::<_, u64>(&DataKey::MerchantId(merchant.clone()))
                {
                    if invoice.merchant_id != merchant_id {
                        matches = false;
                    }
                } else {
                    matches = false;
                }
            }

            if let Some(min_amount) = filter.min_amount {
                if invoice.amount < min_amount as i128 {
                    matches = false;
                }
            }

            if let Some(max_amount) = filter.max_amount {
                if invoice.amount > max_amount as i128 {
                    matches = false;
                }
            }

            if matches {
                invoices.push_back(invoice);
            }
        }
    }

    invoices
}

pub fn pay_invoice_admin(env: &Env, admin_or_manager: &Address, invoice_id: u64) -> Invoice {
    admin_or_manager.require_auth();

    // Check authorization - must have admin or manager role
    let has_admin_role = access_control::has_role(env, admin_or_manager, Role::Admin);
    let has_manager_role = access_control::has_role(env, admin_or_manager, Role::Manager);

    if !has_admin_role && !has_manager_role {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    // Get the invoice
    let mut invoice = get_invoice(env, invoice_id);

    // Verify invoice status is Pending
    if invoice.status != InvoiceStatus::Pending {
        panic_with_error!(env, ContractError::InvalidInvoiceStatus);
    }

    // Get the fee for this token
    use crate::components::admin as admin_component;
    let fee = admin_component::get_fee(env, &invoice.token);

    // Calculate amount to merchant
    let merchant_amount = invoice.amount - fee;

    // Get merchant info to verify it exists
    let merchant = merchant::get_merchant(env, invoice.merchant_id);

    // Transfer tokens from contract to merchant account
    let contract_address = env.current_contract_address();
    let token_client = token::Client::new(env, &invoice.token);

    token_client.transfer(&contract_address, &merchant.address, &merchant_amount);

    // Update invoice status
    invoice.status = InvoiceStatus::Paid;
    invoice.payer = Some(admin_or_manager.clone());
    invoice.date_paid = Some(env.ledger().timestamp());

    env.storage()
        .persistent()
        .set(&DataKey::Invoice(invoice_id), &invoice);

    // Publish paid event
    events::publish_invoice_paid_event(
        env,
        invoice_id,
        invoice.merchant_id,
        admin_or_manager.clone(),
        invoice.amount,
        fee,
        invoice.token.clone(),
        env.ledger().timestamp(),
    );

    invoice
}
