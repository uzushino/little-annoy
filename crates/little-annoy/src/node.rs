use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Node<T, S> {
    Item(Vec<T>),
    Bucket(Vec<i64>),
    Split { plane: S, children: [i64; 2] },
}

impl<T, S> Node<T, S> {
    pub(crate) fn item(&self) -> Option<&[T]> {
        match self {
            Self::Item(vector) => Some(vector),
            _ => None,
        }
    }
}
