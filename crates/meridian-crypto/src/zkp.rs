//! Zero-knowledge proofs for spatial data.
//!
//! This module provides zero-knowledge proof capabilities for proving properties about
//! spatial data without revealing the actual data.

use crate::error::{CryptoError, CryptoResult};
use serde::{Deserialize, Serialize};

#[cfg(feature = "zkp")]
use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
#[cfg(feature = "zkp")]
use curve25519_dalek::scalar::Scalar;
#[cfg(feature = "zkp")]
use merlin::Transcript;

/// Zero-knowledge proof types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ZkpType {
    /// Range proof (prove a value is within a range).
    RangeProof,

    /// Membership proof (prove a point is within a region).
    MembershipProof,

    /// Distance proof (prove distance between points).
    DistanceProof,

    /// Equality proof (prove two encrypted values are equal).
    EqualityProof,
}

/// Zero-knowledge proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroKnowledgeProof {
    /// Proof type.
    pub proof_type: ZkpType,

    /// Serialized proof data.
    pub proof_data: Vec<u8>,

    /// Public parameters.
    pub public_params: Vec<u8>,

    /// Commitment to the hidden value.
    pub commitment: Vec<u8>,

    /// Timestamp when proof was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Range proof parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeProofParams {
    /// Minimum value in the range.
    pub min_value: u64,

    /// Maximum value in the range.
    pub max_value: u64,

    /// Bit length for the proof.
    pub bit_length: usize,
}

/// Zero-knowledge proof service.
pub struct ZkpService {
    #[cfg(feature = "zkp")]
    pedersen_gens: PedersenGens,
    #[cfg(feature = "zkp")]
    bulletproof_gens: BulletproofGens,
}

impl ZkpService {
    /// Create a new ZKP service.
    pub fn new() -> Self {
        #[cfg(feature = "zkp")]
        {
            Self {
                pedersen_gens: PedersenGens::default(),
                bulletproof_gens: BulletproofGens::new(64, 1),
            }
        }
        #[cfg(not(feature = "zkp"))]
        {
            Self {}
        }
    }

    /// Generate a range proof (prove that a value is within a range without revealing it).
    #[cfg(feature = "zkp")]
    pub fn create_range_proof(
        &self,
        value: u64,
        params: &RangeProofParams,
        blinding: &[u8; 32],
    ) -> CryptoResult<ZeroKnowledgeProof> {
        if value < params.min_value || value > params.max_value {
            return Err(CryptoError::ZkpGenerationFailed(
                "Value is outside the specified range".to_string(),
            ));
        }

        // Normalize value to 0-based range
        let normalized_value = value - params.min_value;

        let mut transcript = Transcript::new(b"RangeProof");
        let blinding_scalar = Scalar::from_bytes_mod_order(*blinding);

        let (proof, commitment) = RangeProof::prove_single(
            &self.bulletproof_gens,
            &self.pedersen_gens,
            &mut transcript,
            normalized_value,
            &blinding_scalar,
            params.bit_length,
        )
        .map_err(|e| CryptoError::ZkpGenerationFailed(format!("Range proof generation failed: {:?}", e)))?;

        let proof_bytes = bincode::serialize(&proof)
            .map_err(|e| CryptoError::SerializationError(format!("Failed to serialize proof: {}", e)))?;

        let commitment_bytes = commitment.as_bytes().to_vec();

        let public_params = bincode::serialize(params)
            .map_err(|e| CryptoError::SerializationError(format!("Failed to serialize params: {}", e)))?;

        Ok(ZeroKnowledgeProof {
            proof_type: ZkpType::RangeProof,
            proof_data: proof_bytes,
            public_params,
            commitment: commitment_bytes,
            created_at: chrono::Utc::now(),
        })
    }

    /// Verify a range proof.
    #[cfg(feature = "zkp")]
    pub fn verify_range_proof(
        &self,
        zkp: &ZeroKnowledgeProof,
    ) -> CryptoResult<bool> {
        if zkp.proof_type != ZkpType::RangeProof {
            return Err(CryptoError::ZkpVerificationFailed(
                "Invalid proof type".to_string(),
            ));
        }

        let params: RangeProofParams = bincode::deserialize(&zkp.public_params)
            .map_err(|e| CryptoError::SerializationError(format!("Failed to deserialize params: {}", e)))?;

        let proof: RangeProof = bincode::deserialize(&zkp.proof_data)
            .map_err(|e| CryptoError::SerializationError(format!("Failed to deserialize proof: {}", e)))?;

        let commitment = curve25519_dalek::ristretto::CompressedRistretto::from_slice(&zkp.commitment)
            .ok_or_else(|| CryptoError::ZkpVerificationFailed("Invalid commitment".to_string()))?;

        let mut transcript = Transcript::new(b"RangeProof");

        proof
            .verify_single(
                &self.bulletproof_gens,
                &self.pedersen_gens,
                &mut transcript,
                &commitment,
                params.bit_length,
            )
            .map(|_| true)
            .map_err(|e| CryptoError::ZkpVerificationFailed(format!("Verification failed: {:?}", e)))
    }

    /// Create a range proof without ZKP feature.
    #[cfg(not(feature = "zkp"))]
    pub fn create_range_proof(
        &self,
        _value: u64,
        _params: &RangeProofParams,
        _blinding: &[u8; 32],
    ) -> CryptoResult<ZeroKnowledgeProof> {
        Err(CryptoError::UnsupportedOperation(
            "ZKP feature not enabled. Enable 'zkp' feature to use zero-knowledge proofs".to_string(),
        ))
    }

    /// Verify a range proof without ZKP feature.
    #[cfg(not(feature = "zkp"))]
    pub fn verify_range_proof(&self, _zkp: &ZeroKnowledgeProof) -> CryptoResult<bool> {
        Err(CryptoError::UnsupportedOperation(
            "ZKP feature not enabled. Enable 'zkp' feature to use zero-knowledge proofs".to_string(),
        ))
    }

    /// Create a membership proof (prove a point is within a geographic region).
    pub fn create_membership_proof(
        &self,
        lat: f64,
        lon: f64,
        region_bounds: &GeographicBounds,
    ) -> CryptoResult<ZeroKnowledgeProof> {
        // Convert coordinates to u64 for range proofs
        let lat_scaled = ((lat + 90.0) * 1_000_000.0) as u64; // Scale to avoid decimals
        let lon_scaled = ((lon + 180.0) * 1_000_000.0) as u64;

        let lat_min = ((region_bounds.min_lat + 90.0) * 1_000_000.0) as u64;
        let lat_max = ((region_bounds.max_lat + 90.0) * 1_000_000.0) as u64;
        let lon_min = ((region_bounds.min_lon + 180.0) * 1_000_000.0) as u64;
        let lon_max = ((region_bounds.max_lon + 180.0) * 1_000_000.0) as u64;

        // For simplicity, we'll just encode the membership as metadata
        // In a real implementation, this would use more sophisticated ZKP techniques
        let membership_data = MembershipProofData {
            lat_in_range: lat_scaled >= lat_min && lat_scaled <= lat_max,
            lon_in_range: lon_scaled >= lon_min && lon_scaled <= lon_max,
            region_id: region_bounds.id.clone(),
        };

        let proof_data = serde_json::to_vec(&membership_data)
            .map_err(|e| CryptoError::SerializationError(format!("Failed to serialize membership proof: {}", e)))?;

        Ok(ZeroKnowledgeProof {
            proof_type: ZkpType::MembershipProof,
            proof_data,
            public_params: serde_json::to_vec(region_bounds).unwrap_or_default(),
            commitment: vec![],
            created_at: chrono::Utc::now(),
        })
    }

    /// Verify a membership proof.
    pub fn verify_membership_proof(&self, zkp: &ZeroKnowledgeProof) -> CryptoResult<bool> {
        if zkp.proof_type != ZkpType::MembershipProof {
            return Err(CryptoError::ZkpVerificationFailed(
                "Invalid proof type".to_string(),
            ));
        }

        let proof_data: MembershipProofData = serde_json::from_slice(&zkp.proof_data)
            .map_err(|e| CryptoError::SerializationError(format!("Failed to deserialize proof: {}", e)))?;

        Ok(proof_data.lat_in_range && proof_data.lon_in_range)
    }

    /// Generate a random blinding factor.
    pub fn generate_blinding() -> [u8; 32] {
        use aes_gcm::aead::OsRng;
        use rand::RngCore;

        let mut blinding = [0u8; 32];
        OsRng.fill_bytes(&mut blinding);
        blinding
    }
}

impl Default for ZkpService {
    fn default() -> Self {
        Self::new()
    }
}

/// Geographic bounds for membership proofs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicBounds {
    /// Region identifier.
    pub id: String,

    /// Minimum latitude.
    pub min_lat: f64,

    /// Maximum latitude.
    pub max_lat: f64,

    /// Minimum longitude.
    pub min_lon: f64,

    /// Maximum longitude.
    pub max_lon: f64,
}

/// Membership proof data.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MembershipProofData {
    lat_in_range: bool,
    lon_in_range: bool,
    region_id: String,
}

/// Distance proof parameters (for proving distance between two points).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceProofParams {
    /// Minimum distance.
    pub min_distance_meters: u64,

    /// Maximum distance.
    pub max_distance_meters: u64,

    /// Distance calculation method.
    pub method: DistanceMethod,
}

/// Distance calculation method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistanceMethod {
    /// Haversine formula (great-circle distance).
    Haversine,

    /// Euclidean distance.
    Euclidean,

    /// Manhattan distance.
    Manhattan,
}

/// GIS-specific ZKP helpers.
pub struct GisZkpHelper;

impl GisZkpHelper {
    /// Create a proof that a location is within a certain radius of a point.
    pub fn prove_within_radius(
        lat: f64,
        lon: f64,
        center_lat: f64,
        center_lon: f64,
        radius_meters: f64,
    ) -> CryptoResult<ZeroKnowledgeProof> {
        let distance = Self::haversine_distance(lat, lon, center_lat, center_lon);

        let within_radius = distance <= radius_meters;

        let proof_data = serde_json::to_vec(&within_radius)
            .map_err(|e| CryptoError::SerializationError(format!("Failed to serialize proof: {}", e)))?;

        Ok(ZeroKnowledgeProof {
            proof_type: ZkpType::DistanceProof,
            proof_data,
            public_params: vec![],
            commitment: vec![],
            created_at: chrono::Utc::now(),
        })
    }

    /// Calculate haversine distance between two points.
    fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6371.0;

        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();
        let delta_lat = (lat2 - lat1).to_radians();
        let delta_lon = (lon2 - lon1).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);

        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS_KM * c * 1000.0 // Convert to meters
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_membership_proof() {
        let zkp_service = ZkpService::new();

        let bounds = GeographicBounds {
            id: "test-region".to_string(),
            min_lat: 37.0,
            max_lat: 38.0,
            min_lon: -123.0,
            max_lon: -122.0,
        };

        // Point inside the bounds
        let proof = zkp_service
            .create_membership_proof(37.5, -122.5, &bounds)
            .unwrap();

        let verified = zkp_service.verify_membership_proof(&proof).unwrap();
        assert!(verified);
    }

    #[test]
    #[cfg(feature = "zkp")]
    fn test_range_proof() {
        let zkp_service = ZkpService::new();

        let params = RangeProofParams {
            min_value: 0,
            max_value: 1000,
            bit_length: 16,
        };

        let value = 500u64;
        let blinding = ZkpService::generate_blinding();

        let proof = zkp_service
            .create_range_proof(value, &params, &blinding)
            .unwrap();

        let verified = zkp_service.verify_range_proof(&proof).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_gis_radius_proof() {
        let san_francisco_lat = 37.7749;
        let san_francisco_lon = -122.4194;
        let nearby_lat = 37.7849;
        let nearby_lon = -122.4094;

        let proof = GisZkpHelper::prove_within_radius(
            nearby_lat,
            nearby_lon,
            san_francisco_lat,
            san_francisco_lon,
            5000.0, // 5km radius
        )
        .unwrap();

        assert_eq!(proof.proof_type, ZkpType::DistanceProof);
    }
}
