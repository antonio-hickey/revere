use core::fmt;
use serde::de::Visitor;
use serde::{de, Deserialize, Deserializer};
use smithay_client_toolkit::reexports::protocols::wlr::unstable::layer_shell::v1::client::zwlr_layer_surface_v1;
use std::{env, fs, path::PathBuf};

#[derive(Deserialize)]
pub struct Config {
    pub window: WindowConfig,
}
impl Config {
    /// Find user configuration file, or if not found does default config
    pub fn find() -> Config {
        // Use home directory path + ".config/revere/config.toml"
        let home_dir = env::var("HOME").expect("Your HOME env variable is not setup broooo");
        let mut path = PathBuf::from(home_dir);
        path.push(".config/revere/config.toml");

        // Try to serialize user config file or if running
        // into problems then use the default config
        if let Ok(config) = fs::read_to_string(path) {
            toml::from_str(&config).unwrap_or(Config::default())
        } else {
            Config::default()
        }
    }

    /// Builds a default `Config` instance
    fn default() -> Config {
        Config {
            window: WindowConfig {
                placement: WindowPlacement {
                    x: Placement::Top,
                    y: Placement::Right,
                },
                size: WindowSize {
                    height: 100,
                    width: 200,
                },
                margin: WindowMargin {
                    top: 10,
                    right: 10,
                    bottom: 0,
                    left: 0,
                },
                color: WindowColor {
                    // White background
                    bg: Rgb {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                    },
                    // Black text
                    fg: Rgb {
                        red: 0.0,
                        green: 0.0,
                        blue: 0.0,
                    },
                },
                font_size: 15,
                duration: 3,
            },
        }
    }
}

/// Notification Window configuration
#[derive(Deserialize)]
pub struct WindowConfig {
    /// Where to place the window
    pub placement: WindowPlacement,
    /// How big of a window
    pub size: WindowSize,
    /// How much margin for the window
    pub margin: WindowMargin,
    /// What colors for the window
    pub color: WindowColor,
    /// The window's text size
    /// Default = `15`
    pub font_size: u8,
    /// How long the window is displayed (seconds)
    /// Defualt = `3`
    pub duration: u8,
}

/// Window Placement Configuration
#[derive(Deserialize)]
pub struct WindowPlacement {
    /// x axis placement (Left or Right)
    /// Default = `Right`
    pub x: Placement,
    /// x axis placement (TOP or Bottom)
    /// Default = `Top`
    pub y: Placement,
}

/// Window Placement Options
#[derive(Deserialize)]
pub enum Placement {
    Top,
    Bottom,
    Right,
    Left,
}
impl Placement {
    /// Mask `Placement` as `Anchor` for wayland
    pub fn as_anchor(&self) -> zwlr_layer_surface_v1::Anchor {
        match self {
            Self::Left => zwlr_layer_surface_v1::Anchor::Left,
            Self::Right => zwlr_layer_surface_v1::Anchor::Right,
            Self::Top => zwlr_layer_surface_v1::Anchor::Top,
            Self::Bottom => zwlr_layer_surface_v1::Anchor::Bottom,
        }
    }
}

/// Window Size Configuration
#[derive(Deserialize)]
pub struct WindowSize {
    /// How tall of a window
    /// Default = `100`
    pub height: u32,
    /// How wide of a window
    /// Default = `200`
    pub width: u32,
}

/// Window Margin Configuration
#[derive(Deserialize)]
pub struct WindowMargin {
    /// How much top margin (px)
    /// Default = `10`
    pub top: i32,
    /// How much right margin (px)
    /// Default = `10`
    pub right: i32,
    /// How much bottom margin (px)
    /// Default = `0`
    pub bottom: i32,
    /// How much left margin (px)
    /// Default = `0`
    pub left: i32,
}

/// Window Color Configuration
// TODO: Border Colors?
#[derive(Deserialize)]
pub struct WindowColor {
    /// Background color
    /// Default = `white`
    #[serde(deserialize_with = "hex_to_rgb")]
    pub bg: Rgb,
    /// Foreground color
    /// Default = `black`
    #[serde(deserialize_with = "hex_to_rgb")]
    pub fg: Rgb,
}

/// Color
#[derive(Deserialize)]
pub struct Rgb {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}

/// Custom parser from hex string into rgb struct
fn hex_to_rgb<'de, D>(deserializer: D) -> Result<Rgb, D::Error>
where
    D: Deserializer<'de>,
{
    struct RGBVisitor;

    impl<'de> Visitor<'de> for RGBVisitor {
        type Value = Rgb;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a hex color string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // Turn hex string into integer and
            // shift it around for the rgb values
            let int_val = i32::from_str_radix(&value[1..], 16).map_err(de::Error::custom)?;
            Ok(Rgb {
                red: ((int_val >> 16) & 0xFF) as f64 / 255.0,
                green: ((int_val >> 8) & 0xFF) as f64 / 255.0,
                blue: (int_val & 0xFF) as f64 / 255.0,
            })
        }
    }

    deserializer.deserialize_str(RGBVisitor)
}
