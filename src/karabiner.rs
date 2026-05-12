//! Karabiner JSON rule schema

use serde::Serialize;

/// Group of manipulators with a label
#[derive(Debug, Serialize)]
pub struct Rule {
    pub description: String,
    pub manipulators: Vec<Manipulator>,
}

/// One key-mapping rule. Empty / `None` fields skipped in JSON.
#[derive(Debug, Serialize)]
pub struct Manipulator {
    pub r#type: String,
    pub from: FromKey,
    pub to: Vec<ToEvent>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub to_if_alone: Vec<ToEvent>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub to_after_key_up: Vec<ToEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Parameters>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<Condition>,
}

impl Manipulator {
    pub fn from_key(key: &str) -> Self {
        Self {
            r#type: "basic".into(),
            from: FromKey {
                key_code: key.into(),
                modifiers: FromModifiers {
                    optional: vec!["any".into()],
                },
            },
            to: vec![],
            to_if_alone: vec![],
            to_after_key_up: vec![],
            parameters: None,
            conditions: vec![],
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FromKey {
    pub key_code: String,
    pub modifiers: FromModifiers,
}

#[derive(Debug, Serialize)]
pub struct FromModifiers {
    pub optional: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum ToEvent {
    Key(ToKey),
    Variable(ToVariable),
}

impl ToEvent {
    pub fn set_var(name: impl Into<String>, value: u32) -> Self {
        ToEvent::Variable(ToVariable {
            set_variable: VariableValue {
                name: name.into(),
                value,
            },
        })
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ToKey {
    pub key_code: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ToVariable {
    pub set_variable: VariableValue,
}

#[derive(Debug, Serialize, Clone)]
pub struct VariableValue {
    pub name: String,
    pub value: u32,
}

#[derive(Debug, Serialize)]
pub struct Parameters {
    #[serde(rename = "basic.to_if_alone_timeout_milliseconds")]
    pub to_if_alone_timeout: u32,
}

#[derive(Debug, Serialize)]
pub struct Condition {
    pub r#type: String,
    pub name: String,
    pub value: u32,
}

impl Condition {
    pub fn variable_if(name: impl Into<String>, value: u32) -> Self {
        Self {
            r#type: "variable_if".into(),
            name: name.into(),
            value,
        }
    }
}
