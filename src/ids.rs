//! Typed ids at the module boundaries, so a pane id cannot be passed where a task id belongs.
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, fmt, ops::Deref};

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_string())
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl Deref for $name {
            type Target = str;

            fn deref(&self) -> &str {
                &self.0
            }
        }

        impl Borrow<str> for $name {
            fn borrow(&self) -> &str {
                &self.0
            }
        }

        impl PartialEq<str> for $name {
            fn eq(&self, other: &str) -> bool {
                self.0 == other
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool {
                self.0 == *other
            }
        }
    };
}

id_type!(TaskId);
id_type!(PaneId);
id_type!(WorkspaceId);
id_type!(ListId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_compare_and_format_as_text() {
        assert_eq!(TaskId::from("86a"), "86a");
        assert_eq!(TaskId::from("86a").to_string(), "86a");
        assert_eq!(TaskId::new("86a").as_str(), "86a");
    }

    #[test]
    fn ids_survive_json() {
        let id = PaneId::from("w1:p1");
        let text = serde_json::to_string(&id).unwrap();
        assert_eq!(text, "\"w1:p1\"");
        assert_eq!(serde_json::from_str::<PaneId>(&text).unwrap(), id);
    }

    #[test]
    fn a_map_accepts_a_plain_string_key() {
        let mut map = std::collections::BTreeMap::new();
        map.insert(WorkspaceId::from("w1"), 1);
        assert_eq!(map.get("w1"), Some(&1));
    }
}
