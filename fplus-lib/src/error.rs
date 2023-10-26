use std::{
    fmt::Display,
    pin::Pin,
    task::{Context, Poll},
};

use actix_web::{
    body::{BodySize, MessageBody},
    web::Bytes,
};

#[derive(Debug)]
pub enum LDNApplicationError {
    NewApplicationError(String),
    LoadApplicationError(String),
}

impl Display for LDNApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LDNApplicationError::LoadApplicationError(e) => {
                write!(f, "LoadApplicationError: {}", e)
            }
            LDNApplicationError::NewApplicationError(e) => {
                write!(f, "NewApplicationError: {}", e)
            }
        }
    }
}

impl MessageBody for LDNApplicationError {
    type Error = std::convert::Infallible;

    fn size(&self) -> BodySize {
        match self {
            LDNApplicationError::LoadApplicationError(e) => BodySize::Sized(e.len() as u64),
            LDNApplicationError::NewApplicationError(e) => BodySize::Sized(e.len() as u64),
        }
    }

    fn poll_next(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        match Pin::<&mut LDNApplicationError>::into_inner(self) {
            LDNApplicationError::LoadApplicationError(e) => {
                Poll::Ready(Some(Ok(Bytes::from(e.clone()))))
            }
            LDNApplicationError::NewApplicationError(e) => {
                Poll::Ready(Some(Ok(Bytes::from(e.clone()))))
            }
        }
    }
}
