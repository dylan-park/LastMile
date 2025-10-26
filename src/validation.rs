use crate::error::{AppError, Result};
use rust_decimal::Decimal;

pub fn validate_odometer(start: i32, end: i32) -> Result<()> {
    if end < start {
        return Err(AppError::InvalidOdometer { start, end });
    }
    Ok(())
}

pub fn validate_monetary_value(name: &str, value: &Decimal) -> Result<()> {
    if value < &Decimal::ZERO {
        return Err(AppError::InvalidMonetaryValue(name.to_string()));
    }
    Ok(())
}

pub fn validate_monetary_values(
    earnings: &Decimal,
    tips: &Decimal,
    gas_cost: &Decimal,
) -> Result<()> {
    validate_monetary_value("earnings", earnings)?;
    validate_monetary_value("tips", tips)?;
    validate_monetary_value("gas_cost", gas_cost)?;
    Ok(())
}

pub fn sanitize_notes(notes: Option<String>) -> Option<String> {
    notes.and_then(|n| {
        let trimmed = n.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}
