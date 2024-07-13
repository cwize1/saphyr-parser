use std::{borrow::Cow, fmt::{self, Display}};

use crate::{TScalarStyle, Tag};

/// Contains information for YAML scalar values.
#[derive(Clone, PartialEq, Debug, Eq)]
pub enum ScalarValue<'a> {
    /// A number value.
    Real(Cow<'a, str>),
    /// A string value.
    String(Cow<'a, str>),
    /// A boolean value.
    Boolean(bool),
    /// An integer value.
    Integer(i64),
    /// The null value.
    Null,
    /// Invalid type conversion returns `BadValue`.
    BadValue,
}

impl ScalarValue<'_> {
    /// Converts a [`crate::Event::Scalar`] event data into a scalar value.
    pub fn from_scalar_event(value: String, style: TScalarStyle, _anchor_id: usize, tag: Option<Tag>) -> ScalarValue<'static> {
        if style != TScalarStyle::Plain {
            ScalarValue::String(Cow::Owned(value))
        } else if let Some(Tag {
            ref handle,
            ref suffix,
        }) = tag
        {
            if handle == "tag:yaml.org,2002:" {
                match suffix.as_ref() {
                    "bool" => {
                        // "true" or "false"
                        match value.parse::<bool>() {
                            Err(_) => ScalarValue::BadValue,
                            Ok(v) => ScalarValue::Boolean(v),
                        }
                    }
                    "int" => match value.parse::<i64>() {
                        Err(_) => ScalarValue::BadValue,
                        Ok(v) => ScalarValue::Integer(v),
                    },
                    "float" => match Self::parse_f64(&value) {
                        Some(_) => ScalarValue::Real(Cow::Owned(value)),
                        None => ScalarValue::BadValue,
                    },
                    "null" => match value.as_ref() {
                        "~" | "null" => ScalarValue::Null,
                        _ => ScalarValue::BadValue,
                    },
                    _ => ScalarValue::String(Cow::Owned(value)),
                }
            } else {
                ScalarValue::String(Cow::Owned(value))
            }
        } else {
            // Datatype is not specified, or unrecognized
            ScalarValue::from_string(value)
        }
    }

    /// Convert a reference string to a [`ScalarValue`] node.
    ///
    /// [`ScalarValue`] does not implement [`std::str::FromStr`] since conversion may not fail. This
    /// function falls back to [`ScalarValue::String`] if nothing else matches.
    ///
    /// # Examples
    /// ```
    /// # use saphyr_parser::ScalarValue;
    /// assert!(matches!(ScalarValue::from_str("42"), ScalarValue::Integer(42)));
    /// assert!(matches!(ScalarValue::from_str("0x2A"), ScalarValue::Integer(42)));
    /// assert!(matches!(ScalarValue::from_str("0o52"), ScalarValue::Integer(42)));
    /// assert!(matches!(ScalarValue::from_str("~"), ScalarValue::Null));
    /// assert!(matches!(ScalarValue::from_str("null"), ScalarValue::Null));
    /// assert!(matches!(ScalarValue::from_str("true"), ScalarValue::Boolean(true)));
    /// assert!(matches!(ScalarValue::from_str("3.14"), ScalarValue::Real(_)));
    /// assert!(matches!(ScalarValue::from_str("foo"), ScalarValue::String(_)));
    /// ```
    #[must_use]
    pub fn from_str(v: &str) -> ScalarValue {
        Self::from_cow_str(Cow::Borrowed(v))
    }

    /// Convert a string to a [`ScalarValue`] node.
    pub fn from_string(v: String) -> ScalarValue<'static> {
        Self::from_cow_str(Cow::Owned(v))
    }

    /// Convert a string or a reference string to a [`ScalarValue`] node.
    pub fn from_cow_str<'a>(v: Cow<'a, str>) -> ScalarValue<'a> {
        if let Some(number) = v.strip_prefix("0x") {
            if let Ok(i) = i64::from_str_radix(number, 16) {
                return ScalarValue::Integer(i);
            }
        } else if let Some(number) = v.strip_prefix("0o") {
            if let Ok(i) = i64::from_str_radix(number, 8) {
                return ScalarValue::Integer(i);
            }
        } else if let Some(number) = v.strip_prefix('+') {
            if let Ok(i) = number.parse::<i64>() {
                return ScalarValue::Integer(i);
            }
        }
        match v.as_ref() {
            "~" | "null" => ScalarValue::Null,
            "true" => ScalarValue::Boolean(true),
            "false" => ScalarValue::Boolean(false),
            _ => {
                if let Ok(integer) = v.parse::<i64>() {
                    ScalarValue::Integer(integer)
                } else if Self::parse_f64(v.as_ref()).is_some() {
                    ScalarValue::Real(v)
                } else {
                    ScalarValue::String(v)
                }
            }
        }
    }

    // parse f64 as Core schema
    // See: https://github.com/chyh1990/yaml-rust/issues/51
    fn parse_f64(v: &str) -> Option<f64> {
        match v {
            ".inf" | ".Inf" | ".INF" | "+.inf" | "+.Inf" | "+.INF" => Some(f64::INFINITY),
            "-.inf" | "-.Inf" | "-.INF" => Some(f64::NEG_INFINITY),
            ".nan" | "NaN" | ".NAN" => Some(f64::NAN),
            _ => v.parse::<f64>().ok(),
        }
    }

}

impl Display for ScalarValue<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScalarValue::Real(value) => Display::fmt(value, formatter),
            ScalarValue::String(value) => Display::fmt(value, formatter),
            ScalarValue::Boolean(value) => Display::fmt(value, formatter),
            ScalarValue::Integer(value) => Display::fmt(value, formatter),
            ScalarValue::Null => formatter.write_str("~"),
            ScalarValue::BadValue => formatter.write_str("~"),
        }
    }
}
