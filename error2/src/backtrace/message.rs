use crate::StrId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct Message {
    type_name: StrId,
    display: Box<str>,
}

impl Message {
    pub(super) fn new(type_name: &'static str, display: String) -> Self {
        Self {
            type_name: type_name.into(),
            display: display.into(),
        }
    }

    pub(crate) const fn type_name(&self) -> &StrId {
        &self.type_name
    }

    pub(crate) const fn display(&self) -> &str {
        &self.display
    }
}
