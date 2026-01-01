//! Zero-Trust Security Architecture
//!
//! Implements zero-trust principles for enterprise security:
//! - Never trust, always verify
//! - Assume breach
//! - Verify explicitly
//! - Use least privilege access
//! - Segment access
//!
//! ## Components
//! - Policy Engine: Define and evaluate access policies
//! - Security Context: Request metadata for decision making
//! - Continuous verification: Re-evaluate trust on every request
//!
//! ## NIST Zero Trust Architecture (SP 800-207)
//! This implementation aligns with NIST guidelines for:
//! - Policy-based access control
//! - Context-aware security
//! - Micro-segmentation
//! - Continuous monitoring

pub mod context;
pub mod policy;

pub use context::{RequestContext, SecurityContext};
pub use policy::{AccessDecision, Policy, PolicyEngine, PolicyRule};
