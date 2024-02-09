use crate::*;
use lib::string::String;
use lib::vec::Vec;

mod serde_base64 {
    use super::*;

    pub fn serialize<S: serde::Serializer>(
        v: &[u8; 32],
        s: S,
    ) -> Result<S::Ok, S::Error> {
        s.serialize_str(&b64_encode(v))
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        d: D,
    ) -> Result<[u8; 32], D::Error> {
        let dec: String = serde::Deserialize::deserialize(d)?;
        let dec: Vec<u8> =
            b64_decode(&dec).map_err(|e| serde::de::Error::custom(e))?;
        if dec.len() != 32 {
            return Err(serde::de::Error::custom("invalid length"));
        }
        let mut out = [0; 32];
        out[..].copy_from_slice(&dec[..]);
        Ok(out)
    }
}

/// Assertion that a particular agent is online (or offline).
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AgentInfo {
    /// The context under which this agent is acting.
    /// In Holochain, this is the DNA Hash.
    #[serde(with = "serde_base64")]
    #[serde(rename = "s")]
    pub space: [u8; 32],

    /// The agent identifier. This is an ed25519 public key.
    #[serde(with = "serde_base64")]
    #[serde(rename = "a")]
    pub agent: [u8; 32],

    /// If Some, this is the url at which this agent can be reached.
    /// If None, this is an assertion that the agent has gone offline.
    #[serde(rename = "u")]
    pub url: Option<String>,

    /// The microseconds since the unix epoch at which time this assertion
    /// was signed.
    #[serde(rename = "t")]
    pub signed_at_micros: i64,

    /// The microseconds since the unix epoch at which time this assertion
    /// will no longer be valid.
    #[serde(rename = "e")]
    pub expires_at_micros: i64,

    /// Any additional extra metadata to be included with this assertion.
    #[serde(rename = "x")]
    pub extra: serde_json::Value,
}

impl lib::fmt::Debug for AgentInfo {
    fn fmt(&self, f: &mut lib::fmt::Formatter<'_>) -> lib::fmt::Result {
        let space = b64_encode(&self.space);
        let agent = b64_encode(&self.agent);
        f.debug_struct("AgentInfo")
            .field("space", &space)
            .field("agent", &agent)
            .field("url", &self.url)
            .field("signed_at_micros", &self.signed_at_micros)
            .field("expires_at_micros", &self.expires_at_micros)
            .field("extra", &self.extra)
            .finish()
    }
}

impl AgentInfo {
    /// Output the canonical encoding for use in signing.
    pub fn encode(&self) -> Result<String, BootstrapError> {
        serde_json::to_string(&self).map_err(BootstrapError::from_str)
    }
}

/// Signed assertion that a particular agent is online (or offline).
#[derive(Clone, PartialEq, Eq)]
pub struct AgentInfoSigned {
    /// The decoded assertion data.
    pub agent_info: AgentInfo,

    /// The encoded assertion data.
    pub encoded: String,

    /// The signature over the encoded bytes of assertion data.
    pub signature: [u8; 64],
}

impl lib::fmt::Debug for AgentInfoSigned {
    fn fmt(&self, f: &mut lib::fmt::Formatter<'_>) -> lib::fmt::Result {
        let sig = b64_encode(&self.signature);
        f.debug_struct("AgentInfoSigned")
            .field("agent_info", &self.agent_info)
            .field("encoded", &self.encoded)
            .field("signature", &sig)
            .finish()
    }
}

impl lib::ops::Deref for AgentInfoSigned {
    type Target = AgentInfo;

    fn deref(&self) -> &Self::Target {
        &self.agent_info
    }
}

impl serde::Serialize for AgentInfoSigned {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(serde::Serialize)]
        struct Enc<'lt> {
            e: &'lt String,
            s: String,
        }
        Enc {
            e: &self.encoded,
            s: b64_encode(&self.signature[..]),
        }
        .serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for AgentInfoSigned {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Dec {
            e: String,
            s: String,
        }
        let dec = Dec::deserialize(deserializer)?;
        let Dec { e: encoded, s } = dec;
        let agent_info: AgentInfo = serde_json::from_str(&encoded)
            .map_err(|e| serde::de::Error::custom(e))?;
        let s: Vec<u8> =
            b64_decode(&s).map_err(|e| serde::de::Error::custom(e))?;
        if s.len() != 64 {
            return Err(serde::de::Error::custom("invalid length"));
        }
        let mut signature = [0; 64];
        signature[..].copy_from_slice(&s[..]);
        let this = AgentInfoSigned {
            agent_info,
            encoded,
            signature,
        };
        this.verify().map_err(|e| serde::de::Error::custom(e))?;
        Ok(this)
    }
}

impl AgentInfoSigned {
    /// Construct a new AgentInfoSigned from components.
    pub fn new(
        encoded: String,
        signature: [u8; 64],
    ) -> Result<Self, BootstrapError> {
        let agent_info: AgentInfo =
            serde_json::from_str(&encoded).map_err(BootstrapError::from_str)?;

        let this = Self {
            agent_info,
            encoded,
            signature,
        };

        this.verify()?;

        Ok(this)
    }

    /// Verify the cryptographic signature against the encoded content.
    pub fn verify(&self) -> Result<(), BootstrapError> {
        let vkey = ed25519_dalek::VerifyingKey::from_bytes(&self.agent)
            .map_err(BootstrapError::from_str)?;
        let sig = ed25519_dalek::Signature::from_bytes(&self.signature);
        ed25519_dalek::Verifier::verify(&vkey, self.encoded.as_bytes(), &sig)
            .map_err(BootstrapError::from_str)
    }
}

#[cfg(test)]
mod agent_info_tests {
    use super::*;

    #[test]
    fn agent_info_encode_decode() {
        let skey = ed25519_dalek::SigningKey::from_bytes(&[0xdb; 32]);
        let agent = *skey.verifying_key().as_bytes();
        let orig = AgentInfo {
            space: [1; 32],
            agent: agent,
            url: Some(lib::string::ToString::to_string("test://test")),
            signed_at_micros: 42,
            expires_at_micros: 99,
            extra: serde_json::json!({"test": "apple"}),
        };
        let orig = orig.encode().unwrap();
        let sig = ed25519_dalek::Signer::sign(&skey, orig.as_bytes());
        let orig = AgentInfoSigned::new(orig, sig.to_bytes()).unwrap();
        let enc = serde_json::to_string(&orig).unwrap();
        let dec: AgentInfoSigned = serde_json::from_str(&enc).unwrap();
        assert_eq!(orig, dec);
    }
}
