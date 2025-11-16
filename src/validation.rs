use rust_decimal::Decimal;

use crate::error::{AppError, Result};

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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_validate_odometer_valid() {
        assert!(validate_odometer(100, 150).is_ok());
        assert!(validate_odometer(100, 100).is_ok());
        assert!(validate_odometer(0, 100).is_ok());
    }

    #[test]
    fn test_validate_odometer_invalid() {
        let result = validate_odometer(150, 100);
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::InvalidOdometer { start, end } => {
                assert_eq!(start, 150);
                assert_eq!(end, 100);
            }
            _ => panic!("Expected InvalidOdometer error"),
        }
    }

    #[test]
    fn test_validate_monetary_value_valid() {
        assert!(validate_monetary_value("earnings", &dec!(100.0)).is_ok());
        assert!(validate_monetary_value("tips", &dec!(0.0)).is_ok());
        assert!(validate_monetary_value("gas_cost", &dec!(50.50)).is_ok());
    }

    #[test]
    fn test_validate_monetary_value_invalid() {
        let result = validate_monetary_value("earnings", &dec!(-10.0));
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::InvalidMonetaryValue(name) => {
                assert_eq!(name, "earnings");
            }
            _ => panic!("Expected InvalidMonetaryValue error"),
        }
    }

    #[test]
    fn test_validate_monetary_values_all_valid() {
        let result = validate_monetary_values(&dec!(100.0), &dec!(20.0), &dec!(15.0));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_monetary_values_invalid_earnings() {
        let result = validate_monetary_values(&dec!(-10.0), &dec!(20.0), &dec!(15.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_monetary_values_invalid_tips() {
        let result = validate_monetary_values(&dec!(100.0), &dec!(-5.0), &dec!(15.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_monetary_values_invalid_gas() {
        let result = validate_monetary_values(&dec!(100.0), &dec!(20.0), &dec!(-15.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_notes_normal() {
        assert_eq!(
            sanitize_notes(Some("Regular notes".to_string())),
            Some("Regular notes".to_string())
        );
    }

    #[test]
    fn test_sanitize_notes_with_whitespace() {
        assert_eq!(
            sanitize_notes(Some("  Notes with spaces  ".to_string())),
            Some("Notes with spaces".to_string())
        );
    }

    #[test]
    fn test_sanitize_notes_empty_string() {
        assert_eq!(sanitize_notes(Some("".to_string())), None);
    }

    #[test]
    fn test_sanitize_notes_only_whitespace() {
        assert_eq!(sanitize_notes(Some("   ".to_string())), None);
        assert_eq!(sanitize_notes(Some("\n\t".to_string())), None);
    }

    #[test]
    fn test_sanitize_notes_none() {
        assert_eq!(sanitize_notes(None), None);
    }

    #[test]
    fn test_sanitize_notes_multiline() {
        assert_eq!(
            sanitize_notes(Some("  Line 1\nLine 2  ".to_string())),
            Some("Line 1\nLine 2".to_string())
        );
    }
}
