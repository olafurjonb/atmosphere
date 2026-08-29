use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JaffScene {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub layout: Option<String>,
    pub objects: Vec<JaffObject>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JaffObject {
    pub metadata: Metadata,
    pub temporal: TemporalControl,
    pub spatial: SpatialTrajectory,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Metadata {
    pub title: String,
    pub source_sound_file: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TemporalControl {
    pub start_offset: f64,
    pub end_offset: Option<f64>,
    #[serde(rename = "loop")]
    pub loop_sound: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpatialTrajectory {
    #[serde(rename = "startX")]
    pub start_x: f64,
    #[serde(rename = "startY")]
    pub start_y: f64,
    #[serde(rename = "startZ")]
    pub start_z: f64,
    #[serde(default)]
    pub xformula: Option<String>,
    #[serde(default)]
    pub yformula: Option<String>,
    #[serde(default)]
    pub zformula: Option<String>,
    #[serde(default)]
    pub volume: Option<String>,
    #[serde(default, rename = "proximity_bass_boost")]
    pub proximity_bass_boost: bool,
}

impl JaffScene {
    /// Deserializes a JAFF scene from a JSON string.
    /// Supports both a full `JaffScene` object or a raw array of `JaffObject`s.
    pub fn from_json_str(json_str: &str) -> Result<Self, serde_json::Error> {
        // Try parsing as JaffScene first
        if let Ok(scene) = serde_json::from_str::<JaffScene>(json_str) {
            return Ok(scene);
        }

        // If that fails, try parsing as a flat Vec<JaffObject>
        let objects = serde_json::from_str::<Vec<JaffObject>>(json_str)?;
        Ok(JaffScene {
            title: None,
            layout: None,
            objects,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_scene() {
        let json = r#"{
            "title": "Ambient Test Scene",
            "objects": [
                {
                    "metadata": {
                        "title": "Bird Chirp",
                        "source_sound_file": "assets/bird.wav"
                    },
                    "temporal": {
                        "start_offset": 1.5,
                        "end_offset": 5.0,
                        "loop": true
                    },
                    "spatial": {
                        "startX": -0.5,
                        "startY": 0.5,
                        "startZ": 0.1,
                        "xformula": "0.1 * sin(t)",
                        "yformula": "0.2 * cos(t)",
                        "zformula": "0.0",
                        "volume": "1.0"
                    }
                }
            ]
        }"#;

        let scene = JaffScene::from_json_str(json).unwrap();
        assert_eq!(scene.title.as_deref(), Some("Ambient Test Scene"));
        assert_eq!(scene.objects.len(), 1);
        
        let obj = &scene.objects[0];
        assert_eq!(obj.metadata.title, "Bird Chirp");
        assert_eq!(obj.temporal.start_offset, 1.5);
        assert_eq!(obj.temporal.end_offset, Some(5.0));
        assert!(obj.temporal.loop_sound);
        assert_eq!(obj.spatial.start_x, -0.5);
        assert_eq!(obj.spatial.xformula.as_deref(), Some("0.1 * sin(t)"));
    }

    #[test]
    fn test_parse_flat_array() {
        let json = r#"[
            {
                "metadata": {
                    "title": "Siren",
                    "source_sound_file": "assets/siren.wav"
                },
                "temporal": {
                    "start_offset": 0.0,
                    "end_offset": null,
                    "loop": false
                },
                "spatial": {
                    "startX": 0.0,
                    "startY": 0.0,
                    "startZ": 1.0
                }
            }
        ]"#;

        let scene = JaffScene::from_json_str(json).unwrap();
        assert!(scene.title.is_none());
        assert_eq!(scene.objects.len(), 1);
        let obj = &scene.objects[0];
        assert_eq!(obj.metadata.title, "Siren");
        assert_eq!(obj.temporal.end_offset, None);
        assert!(!obj.temporal.loop_sound);
    }
}
