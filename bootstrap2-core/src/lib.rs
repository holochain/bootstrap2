#![deny(missing_docs)]
#![deny(unsafe_code)]
#![allow(unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]
//! Bootstrap2 core types.

#[cfg(not(feature = "std"))]
extern crate alloc;

/// Re-exported library to deal with no_std diffs.
pub mod lib {
    use crate::*;

    #[cfg(feature = "std")]
    pub use ::std::error::Error;

    /// Stand-in Error Trait, given we are not including "std".
    #[cfg(not(feature = "std"))]
    pub trait Error: ::core::fmt::Debug + ::core::fmt::Display {
        /// The underlying cause of this error, if any.
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            None
        }
    }

    #[cfg(feature = "std")]
    pub use ::std::*;

    #[cfg(not(feature = "std"))]
    pub use ::core::*;
    #[cfg(not(feature = "std"))]
    pub use ::alloc::{string, vec};
}

/// Bootstrap error type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapError {
    /// Generic string-based error type.
    Error(lib::string::String),
}

impl lib::Error for BootstrapError {}

impl lib::fmt::Display for BootstrapError {
    fn fmt(&self, f: &mut lib::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Error(err) => err.fmt(f),
        }
    }
}

impl BootstrapError {
    /// Construct a bootstrap error from something string-like.
    pub fn from_str<S: lib::fmt::Display>(s: S) -> Self {
        use lib::string::ToString;
        Self::Error(s.to_string())
    }
}

fn b64_encode(b: &[u8]) -> lib::string::String {
    use base64::prelude::*;
    BASE64_URL_SAFE_NO_PAD.encode(b)
}

fn b64_decode(s: &str) -> Result<lib::vec::Vec<u8>, BootstrapError> {
    use base64::prelude::*;
    BASE64_URL_SAFE_NO_PAD
        .decode(s)
        .map_err(BootstrapError::from_str)
}

mod agent_info;
pub use agent_info::*;
