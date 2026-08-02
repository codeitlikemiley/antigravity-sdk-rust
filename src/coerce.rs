//! Coercion of model-supplied tool arguments to the types a tool declared.
//!
//! Models routinely send `"3"` where a schema says `integer`, `"true"` where it
//! says `boolean`, or a bare value where it says `array`. Passing those through
//! meant a tool's `serde` deserialization failed and the model was told its
//! *tool* had broken, when the argument was one conversion away from valid.
//!
//! Only unambiguous conversions are performed. Anything else is left exactly as
//! it arrived, so a genuine type error still surfaces as one.

use serde_json::Value;

/// Coerces `args` in place against `schema`, a JSON Schema for the tool's
/// parameters.
///
/// A schema that cannot be parsed, or that describes something other than an
/// object, leaves the arguments untouched.
pub fn coerce_arguments(args: &mut Value, schema: &str) {
    let Ok(schema) = serde_json::from_str::<Value>(schema) else {
        return;
    };
    coerce_value(args, &schema);
}

fn coerce_value(value: &mut Value, schema: &Value) {
    match declared_type(schema) {
        Some("object") => {
            let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
                return;
            };
            // A JSON object arriving as a string is the one whole-value
            // conversion worth doing here; after it, fall through to the
            // per-property pass.
            if let Some(text) = value.as_str()
                && let Ok(parsed) = serde_json::from_str::<Value>(text)
                && parsed.is_object()
            {
                *value = parsed;
            }
            let Some(fields) = value.as_object_mut() else {
                return;
            };
            for (name, property_schema) in properties {
                if let Some(field) = fields.get_mut(name) {
                    coerce_value(field, property_schema);
                }
            }
        }
        Some("array") => {
            if let Some(text) = value.as_str()
                && let Ok(parsed) = serde_json::from_str::<Value>(text)
                && parsed.is_array()
            {
                *value = parsed;
            }
            if let Some(items_schema) = schema.get("items")
                && let Some(items) = value.as_array_mut()
            {
                for item in items {
                    coerce_value(item, items_schema);
                }
            }
        }
        Some(kind @ ("integer" | "number" | "boolean" | "string")) => coerce_scalar(value, kind),
        _ => {}
    }
}

fn coerce_scalar(value: &mut Value, kind: &str) {
    let Some(text) = value.as_str().map(str::trim).map(str::to_string) else {
        // A number or bool where a string was asked for: render it rather than
        // fail. The reverse (`3` for `integer`) is already correct.
        if kind == "string" && (value.is_number() || value.is_boolean()) {
            *value = Value::String(value.to_string());
        }
        return;
    };

    match kind {
        "integer" => {
            if let Ok(n) = text.parse::<i64>() {
                *value = Value::from(n);
            }
        }
        "number" => {
            if let Ok(n) = text.parse::<f64>()
                && let Some(n) = serde_json::Number::from_f64(n)
            {
                *value = Value::Number(n);
            }
        }
        "boolean" => match text.to_ascii_lowercase().as_str() {
            "true" => *value = Value::Bool(true),
            "false" => *value = Value::Bool(false),
            _ => {}
        },
        _ => {}
    }
}

/// The schema's declared type, treating `["string", "null"]` as `string`.
fn declared_type(schema: &Value) -> Option<&str> {
    match schema.get("type")? {
        Value::String(s) => Some(s.as_str()),
        Value::Array(members) => members
            .iter()
            .filter_map(Value::as_str)
            .find(|s| *s != "null"),
        _ => None,
    }
}

/// Convenience wrapper: coerce and return.
#[must_use]
pub fn coerced(mut args: Value, schema: &str) -> Value {
    coerce_arguments(&mut args, schema);
    args
}

#[cfg(test)]
mod tests {
    use super::coerced;
    use serde_json::json;

    const SCHEMA: &str = r#"{
        "type": "object",
        "properties": {
            "limit":   {"type": "integer"},
            "ratio":   {"type": "number"},
            "dry_run": {"type": "boolean"},
            "label":   {"type": "string"},
            "tags":    {"type": "array", "items": {"type": "integer"}},
            "nested":  {"type": "object", "properties": {"deep": {"type": "boolean"}}}
        }
    }"#;

    #[test]
    fn strings_become_the_declared_scalar() {
        let out = coerced(
            json!({"limit": "3", "ratio": "0.5", "dry_run": "TRUE"}),
            SCHEMA,
        );
        assert_eq!(out["limit"], json!(3));
        assert_eq!(out["ratio"], json!(0.5));
        assert_eq!(out["dry_run"], json!(true));
    }

    #[test]
    fn a_number_becomes_a_string_when_one_was_asked_for() {
        let out = coerced(json!({"label": 7}), SCHEMA);
        assert_eq!(out["label"], json!("7"));
    }

    #[test]
    fn arrays_and_objects_arriving_as_json_text_are_parsed() {
        let out = coerced(
            json!({"tags": "[\"1\", \"2\"]", "nested": "{\"deep\": \"false\"}"}),
            SCHEMA,
        );
        assert_eq!(out["tags"], json!([1, 2]), "items coerce too");
        assert_eq!(out["nested"]["deep"], json!(false));
    }

    /// A value that is not one conversion away from valid must arrive
    /// unchanged, so a real type error still reads as one.
    #[test]
    fn nonsense_is_left_alone() {
        let out = coerced(json!({"limit": "not a number", "dry_run": "maybe"}), SCHEMA);
        assert_eq!(out["limit"], json!("not a number"));
        assert_eq!(out["dry_run"], json!("maybe"));
    }

    #[test]
    fn unknown_keys_and_broken_schemas_pass_through() {
        let out = coerced(json!({"surprise": "5"}), SCHEMA);
        assert_eq!(out["surprise"], json!("5"));
        assert_eq!(
            coerced(json!({"limit": "3"}), "not json")["limit"],
            json!("3")
        );
    }

    #[test]
    fn a_nullable_type_still_coerces() {
        let schema = r#"{"type":"object","properties":{"n":{"type":["integer","null"]}}}"#;
        assert_eq!(coerced(json!({"n": "12"}), schema)["n"], json!(12));
    }
}
