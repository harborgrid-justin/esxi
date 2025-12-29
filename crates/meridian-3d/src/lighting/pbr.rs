//! Physically Based Rendering (PBR) material system

use glam::{Vec3, Vec4};
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use wgpu::{Device, Texture};

/// PBR material properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialProperties {
    /// Base color / albedo
    pub base_color: Vec4,

    /// Metallic factor (0 = dielectric, 1 = metal)
    pub metallic: f32,

    /// Roughness factor (0 = smooth, 1 = rough)
    pub roughness: f32,

    /// Index of refraction (for dielectrics)
    pub ior: f32,

    /// Emissive color
    pub emissive: Vec3,

    /// Emissive strength
    pub emissive_strength: f32,

    /// Normal map strength
    pub normal_strength: f32,

    /// Occlusion strength
    pub occlusion_strength: f32,

    /// Alpha mode
    pub alpha_mode: AlphaMode,

    /// Alpha cutoff (for alpha mask mode)
    pub alpha_cutoff: f32,
}

impl Default for MaterialProperties {
    fn default() -> Self {
        Self {
            base_color: Vec4::new(0.8, 0.8, 0.8, 1.0),
            metallic: 0.0,
            roughness: 0.5,
            ior: 1.5, // Typical for glass/plastic
            emissive: Vec3::ZERO,
            emissive_strength: 0.0,
            normal_strength: 1.0,
            occlusion_strength: 1.0,
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
        }
    }
}

/// Alpha rendering mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlphaMode {
    /// Fully opaque
    Opaque,
    /// Alpha masking (binary transparency)
    Mask,
    /// Alpha blending
    Blend,
}

/// Complete PBR material with textures
pub struct PbrMaterial {
    /// Material name
    name: String,

    /// Material properties
    properties: MaterialProperties,

    /// Albedo/base color texture
    albedo_texture: Option<DynamicImage>,

    /// Metallic-roughness texture (metallic in B, roughness in G)
    metallic_roughness_texture: Option<DynamicImage>,

    /// Normal map
    normal_texture: Option<DynamicImage>,

    /// Emissive texture
    emissive_texture: Option<DynamicImage>,

    /// Ambient occlusion texture
    occlusion_texture: Option<DynamicImage>,

    /// GPU textures
    gpu_textures: GpuTextures,
}

/// GPU texture handles
#[derive(Default)]
struct GpuTextures {
    albedo: Option<Texture>,
    metallic_roughness: Option<Texture>,
    normal: Option<Texture>,
    emissive: Option<Texture>,
    occlusion: Option<Texture>,
}

impl PbrMaterial {
    /// Create a new PBR material
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            properties: MaterialProperties::default(),
            albedo_texture: None,
            metallic_roughness_texture: None,
            normal_texture: None,
            emissive_texture: None,
            occlusion_texture: None,
            gpu_textures: GpuTextures::default(),
        }
    }

    /// Create a metal material
    pub fn metal(name: impl Into<String>, base_color: Vec3, roughness: f32) -> Self {
        Self {
            name: name.into(),
            properties: MaterialProperties {
                base_color: Vec4::new(base_color.x, base_color.y, base_color.z, 1.0),
                metallic: 1.0,
                roughness,
                ..Default::default()
            },
            ..Self::new("")
        }
    }

    /// Create a dielectric material
    pub fn dielectric(name: impl Into<String>, base_color: Vec3, roughness: f32) -> Self {
        Self {
            name: name.into(),
            properties: MaterialProperties {
                base_color: Vec4::new(base_color.x, base_color.y, base_color.z, 1.0),
                metallic: 0.0,
                roughness,
                ..Default::default()
            },
            ..Self::new("")
        }
    }

    /// Create a glass material
    pub fn glass(name: impl Into<String>, color: Vec3, roughness: f32, ior: f32) -> Self {
        Self {
            name: name.into(),
            properties: MaterialProperties {
                base_color: Vec4::new(color.x, color.y, color.z, 0.1),
                metallic: 0.0,
                roughness,
                ior,
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            },
            ..Self::new("")
        }
    }

    /// Create an emissive material
    pub fn emissive(name: impl Into<String>, color: Vec3, strength: f32) -> Self {
        Self {
            name: name.into(),
            properties: MaterialProperties {
                base_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                emissive: color,
                emissive_strength: strength,
                ..Default::default()
            },
            ..Self::new("")
        }
    }

    /// Get material name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get properties
    pub fn properties(&self) -> &MaterialProperties {
        &self.properties
    }

    /// Get mutable properties
    pub fn properties_mut(&mut self) -> &mut MaterialProperties {
        &mut self.properties
    }

    /// Set albedo texture
    pub fn set_albedo_texture(&mut self, texture: DynamicImage) {
        self.albedo_texture = Some(texture);
    }

    /// Set metallic-roughness texture
    pub fn set_metallic_roughness_texture(&mut self, texture: DynamicImage) {
        self.metallic_roughness_texture = Some(texture);
    }

    /// Set normal map
    pub fn set_normal_texture(&mut self, texture: DynamicImage) {
        self.normal_texture = Some(texture);
    }

    /// Set emissive texture
    pub fn set_emissive_texture(&mut self, texture: DynamicImage) {
        self.emissive_texture = Some(texture);
    }

    /// Set occlusion texture
    pub fn set_occlusion_texture(&mut self, texture: DynamicImage) {
        self.occlusion_texture = Some(texture);
    }

    /// Initialize GPU resources
    pub fn init_gpu_resources(&mut self, device: &Device) {
        // Upload textures to GPU
        // This would create wgpu::Texture from DynamicImage
        // For now, just a placeholder
    }

    /// Calculate F0 (Fresnel reflectance at normal incidence)
    pub fn calculate_f0(&self) -> Vec3 {
        if self.properties.metallic > 0.5 {
            // Metallic: F0 = base_color
            Vec3::new(
                self.properties.base_color.x,
                self.properties.base_color.y,
                self.properties.base_color.z,
            )
        } else {
            // Dielectric: F0 from IOR
            let f0_scalar = ((1.0 - self.properties.ior) / (1.0 + self.properties.ior)).powi(2);
            Vec3::splat(f0_scalar)
        }
    }
}

/// Material presets
pub struct MaterialPresets;

impl MaterialPresets {
    /// Concrete material
    pub fn concrete() -> PbrMaterial {
        PbrMaterial::dielectric("Concrete", Vec3::new(0.5, 0.5, 0.5), 0.9)
    }

    /// Brick material
    pub fn brick() -> PbrMaterial {
        PbrMaterial::dielectric("Brick", Vec3::new(0.6, 0.3, 0.2), 0.8)
    }

    /// Wood material
    pub fn wood() -> PbrMaterial {
        PbrMaterial::dielectric("Wood", Vec3::new(0.4, 0.25, 0.15), 0.7)
    }

    /// Glass material
    pub fn glass() -> PbrMaterial {
        PbrMaterial::glass("Glass", Vec3::new(0.95, 0.95, 0.95), 0.0, 1.5)
    }

    /// Chrome metal
    pub fn chrome() -> PbrMaterial {
        PbrMaterial::metal("Chrome", Vec3::new(0.7, 0.7, 0.7), 0.1)
    }

    /// Gold metal
    pub fn gold() -> PbrMaterial {
        PbrMaterial::metal("Gold", Vec3::new(1.0, 0.85, 0.57), 0.2)
    }

    /// Copper metal
    pub fn copper() -> PbrMaterial {
        PbrMaterial::metal("Copper", Vec3::new(0.95, 0.64, 0.54), 0.3)
    }

    /// Iron metal
    pub fn iron() -> PbrMaterial {
        PbrMaterial::metal("Iron", Vec3::new(0.56, 0.57, 0.58), 0.5)
    }

    /// Plastic material
    pub fn plastic() -> PbrMaterial {
        PbrMaterial::dielectric("Plastic", Vec3::new(0.8, 0.8, 0.8), 0.3)
    }

    /// Rubber material
    pub fn rubber() -> PbrMaterial {
        PbrMaterial::dielectric("Rubber", Vec3::new(0.2, 0.2, 0.2), 0.9)
    }
}

/// PBR lighting calculations
pub struct PbrLighting;

impl PbrLighting {
    /// Calculate diffuse BRDF (Lambertian)
    pub fn diffuse_brdf(albedo: Vec3) -> Vec3 {
        albedo / std::f32::consts::PI
    }

    /// Calculate Fresnel-Schlick approximation
    pub fn fresnel_schlick(cos_theta: f32, f0: Vec3) -> Vec3 {
        f0 + (Vec3::ONE - f0) * (1.0 - cos_theta).powi(5)
    }

    /// Calculate GGX normal distribution function
    pub fn distribution_ggx(normal: Vec3, halfway: Vec3, roughness: f32) -> f32 {
        let a = roughness * roughness;
        let a2 = a * a;
        let n_dot_h = normal.dot(halfway).max(0.0);
        let n_dot_h2 = n_dot_h * n_dot_h;

        let denom = n_dot_h2 * (a2 - 1.0) + 1.0;
        let denom = std::f32::consts::PI * denom * denom;

        a2 / denom
    }

    /// Calculate Smith's geometry function
    pub fn geometry_smith(normal: Vec3, view: Vec3, light: Vec3, roughness: f32) -> f32 {
        let n_dot_v = normal.dot(view).max(0.0);
        let n_dot_l = normal.dot(light).max(0.0);

        let ggx1 = Self::geometry_schlick_ggx(n_dot_v, roughness);
        let ggx2 = Self::geometry_schlick_ggx(n_dot_l, roughness);

        ggx1 * ggx2
    }

    /// Calculate Schlick-GGX geometry function
    fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
        let r = roughness + 1.0;
        let k = (r * r) / 8.0;

        n_dot_v / (n_dot_v * (1.0 - k) + k)
    }

    /// Calculate full PBR lighting for a point
    pub fn calculate_lighting(
        normal: Vec3,
        view: Vec3,
        light: Vec3,
        light_color: Vec3,
        albedo: Vec3,
        metallic: f32,
        roughness: f32,
        f0: Vec3,
    ) -> Vec3 {
        let halfway = (view + light).normalize();

        // Cook-Torrance BRDF
        let ndf = Self::distribution_ggx(normal, halfway, roughness);
        let g = Self::geometry_smith(normal, view, light, roughness);
        let f = Self::fresnel_schlick(halfway.dot(view).max(0.0), f0);

        let numerator = ndf * g * f;
        let denominator = 4.0 * normal.dot(view).max(0.0) * normal.dot(light).max(0.0) + 0.0001;
        let specular = numerator / denominator;

        // Diffuse
        let k_d = (Vec3::ONE - f) * (1.0 - metallic);
        let diffuse = k_d * Self::diffuse_brdf(albedo);

        // Final lighting
        let n_dot_l = normal.dot(light).max(0.0);
        (diffuse + specular) * light_color * n_dot_l
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_creation() {
        let material = PbrMaterial::new("Test");
        assert_eq!(material.name(), "Test");
        assert_eq!(material.properties().metallic, 0.0);
    }

    #[test]
    fn test_metal_material() {
        let gold = PbrMaterial::metal("Gold", Vec3::new(1.0, 0.85, 0.57), 0.2);
        assert_eq!(gold.properties().metallic, 1.0);
        assert_eq!(gold.properties().roughness, 0.2);
    }

    #[test]
    fn test_dielectric_material() {
        let plastic = PbrMaterial::dielectric("Plastic", Vec3::new(0.8, 0.8, 0.8), 0.3);
        assert_eq!(plastic.properties().metallic, 0.0);
        assert_eq!(plastic.properties().roughness, 0.3);
    }

    #[test]
    fn test_glass_material() {
        let glass = PbrMaterial::glass("Glass", Vec3::ONE, 0.0, 1.5);
        assert_eq!(glass.properties().ior, 1.5);
        assert_eq!(glass.properties().alpha_mode, AlphaMode::Blend);
    }

    #[test]
    fn test_material_presets() {
        let concrete = MaterialPresets::concrete();
        let gold = MaterialPresets::gold();
        let glass = MaterialPresets::glass();

        assert_eq!(concrete.properties().metallic, 0.0);
        assert_eq!(gold.properties().metallic, 1.0);
        assert_eq!(glass.properties().alpha_mode, AlphaMode::Blend);
    }

    #[test]
    fn test_f0_calculation() {
        let metal = PbrMaterial::metal("Metal", Vec3::new(0.7, 0.7, 0.7), 0.1);
        let f0 = metal.calculate_f0();
        assert!((f0.x - 0.7).abs() < 0.01);

        let dielectric = PbrMaterial::dielectric("Dielectric", Vec3::new(0.8, 0.8, 0.8), 0.5);
        let f0 = dielectric.calculate_f0();
        assert!(f0.x < 0.1); // Should be low for dielectric
    }

    #[test]
    fn test_fresnel_schlick() {
        let f0 = Vec3::splat(0.04);
        let fresnel = PbrLighting::fresnel_schlick(0.5, f0);
        assert!(fresnel.x >= f0.x);
        assert!(fresnel.x <= 1.0);
    }
}
