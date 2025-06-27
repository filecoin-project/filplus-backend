use std::{
    fmt::Display,
    pin::Pin,
    task::{Context, Poll},
};

use actix_web::{
    body::{BodySize, MessageBody},
    web::Bytes,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum LDNError {
    New(String),
    Load(String),
}

impl Display for LDNError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LDNError::Load(e) => {
                write!(f, "Load: {e}")
            }
            LDNError::New(e) => {
                write!(f, "New: {e}")
            }
        }
    }
}

impl MessageBody for LDNError {
    type Error = std::convert::Infallible;

    fn size(&self) -> BodySize {
        match self {
            LDNError::Load(e) => BodySize::Sized(e.len() as u64),
            LDNError::New(e) => BodySize::Sized(e.len() as u64),
        }
    }

    fn poll_next(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        match Pin::<&mut LDNError>::into_inner(self) {
            LDNError::Load(e) => Poll::Ready(Some(Ok(Bytes::from(e.clone())))),
            LDNError::New(e) => Poll::Ready(Some(Ok(Bytes::from(e.clone())))),
        }
    }
}
