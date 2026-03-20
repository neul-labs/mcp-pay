use jsonschema::Validator;
use serde_json::Value;
use thiserror::Error;

/// Errors that can occur during manifest validation.
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid value for field '{field}': {reason}")]
    InvalidValue { field: String, reason: String },
}

/// Validate a JSON value against the mcp-pay schema.
pub fn validate_manifest(value: &Value) -> Result<(), ValidationError> {
    // Basic structural validation
    let obj = value
        .as_object()
        .ok_or_else(|| ValidationError::SchemaValidation("Root must be an object".into()))?;

    // Check required fields
    if !obj.contains_key("mcp_pay") {
        return Err(ValidationError::MissingField("mcp_pay".into()));
    }

    if !obj.contains_key("pricing") {
        return Err(ValidationError::MissingField("pricing".into()));
    }

    if !obj.contains_key("accepts") {
        return Err(ValidationError::MissingField("accepts".into()));
    }

    // Validate mcp_pay version format
    if let Some(version) = obj.get("mcp_pay").and_then(|v| v.as_str()) {
        if !version
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.')
        {
            return Err(ValidationError::InvalidValue {
                field: "mcp_pay".into(),
                reason: "Version must be in format X.Y".into(),
            });
        }
    }

    // Validate accepts is an array
    if !obj
        .get("accepts")
        .map(|v| v.is_array())
        .unwrap_or(false)
    {
        return Err(ValidationError::InvalidValue {
            field: "accepts".into(),
            reason: "Must be an array".into(),
        });
    }

    // Validate each payment rail
    if let Some(accepts) = obj.get("accepts").and_then(|v| v.as_array()) {
        for (i, rail) in accepts.iter().enumerate() {
            validate_payment_rail(rail, i)?;
        }
    }

    Ok(())
}

fn validate_payment_rail(rail: &Value, index: usize) -> Result<(), ValidationError> {
    let obj = rail.as_object().ok_or_else(|| ValidationError::InvalidValue {
        field: format!("accepts[{}]", index),
        reason: "Rail must be an object".into(),
    })?;

    // Rail type is required
    if !obj.contains_key("rail") {
        return Err(ValidationError::MissingField(format!(
            "accepts[{}].rail",
            index
        )));
    }

    // Validate rail type
    if let Some(rail_type) = obj.get("rail").and_then(|v| v.as_str()) {
        let valid_types = ["x402", "mpp", "lightning", "card", "ach", "custom"];
        if !valid_types.contains(&rail_type) {
            return Err(ValidationError::InvalidValue {
                field: format!("accepts[{}].rail", index),
                reason: format!(
                    "Unknown rail type '{}'. Valid types: {:?}",
                    rail_type, valid_types
                ),
            });
        }
    }

    Ok(())
}

/// Create a JSON Schema validator for mcp-pay manifests.
pub fn create_validator() -> Result<Validator, ValidationError> {
    let schema = serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": ["mcp_pay", "pricing", "accepts"],
        "properties": {
            "mcp_pay": {
                "type": "string",
                "pattern": "^\\d+\\.\\d+$"
            },
            "server_card": {
                "type": "string"
            },
            "pricing": {
                "type": "object"
            },
            "accepts": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["rail"],
                    "properties": {
                        "rail": {
                            "type": "string",
                            "enum": ["x402", "mpp", "lightning", "card", "ach", "custom"]
                        }
                    }
                }
            }
        }
    });

    Validator::new(&schema).map_err(|e| ValidationError::SchemaValidation(e.to_string()))
}

/// Validate using JSON Schema (stricter validation).
pub fn validate_with_schema(value: &Value) -> Result<(), ValidationError> {
    let validator = create_validator()?;

    if !validator.is_valid(value) {
        // Collect validation errors
        let errors: Vec<String> = validator
            .iter_errors(value)
            .map(|e| e.to_string())
            .collect();
        return Err(ValidationError::SchemaValidation(errors.join("; ")));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_manifest() {
        let json = serde_json::json!({
            "mcp_pay": "0.1",
            "pricing": {},
            "accepts": [
                {"rail": "x402", "network": "eip155:8453"}
            ]
        });

        assert!(validate_manifest(&json).is_ok());
    }

    #[test]
    fn test_missing_mcp_pay() {
        let json = serde_json::json!({
            "pricing": {},
            "accepts": []
        });

        let err = validate_manifest(&json).unwrap_err();
        assert!(matches!(err, ValidationError::MissingField(f) if f == "mcp_pay"));
    }

    #[test]
    fn test_invalid_rail_type() {
        let json = serde_json::json!({
            "mcp_pay": "0.1",
            "pricing": {},
            "accepts": [
                {"rail": "invalid_rail"}
            ]
        });

        let err = validate_manifest(&json).unwrap_err();
        assert!(matches!(err, ValidationError::InvalidValue { .. }));
    }

    #[test]
    fn test_schema_validation() {
        let json = serde_json::json!({
            "mcp_pay": "0.1",
            "pricing": {},
            "accepts": []
        });

        assert!(validate_with_schema(&json).is_ok());
    }
}
