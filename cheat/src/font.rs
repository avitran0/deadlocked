use std::{fmt::Display, sync::Arc};

use egui::{FontData, FontDefinitions, FontFamily};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

const ICONS: &[u8] = include_bytes!("../assets/Icons.ttf");

#[derive(Debug, Clone, Default, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum Font {
    DMSans,
    #[default]
    FiraSans,
    Inter,
    Nunito,
    Ubuntu,
}

impl Font {
    pub fn data(&self) -> &'static [u8] {
        match self {
            Self::DMSans => include_bytes!("../assets/DMSans.ttf"),
            Self::FiraSans => include_bytes!("../assets/FiraSans.ttf"),
            Self::Inter => include_bytes!("../assets/Inter.ttf"),
            Self::Nunito => include_bytes!("../assets/Nunito.ttf"),
            Self::Ubuntu => include_bytes!("../assets/Ubuntu.ttf"),
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            Self::DMSans => "dm_sans",
            Self::FiraSans => "fira_sans",
            Self::Inter => "inter",
            Self::Nunito => "nunito",
            Self::Ubuntu => "ubuntu",
        }
    }

    fn register(ctx: &egui::Context, primary: &str) {
        let mut font_defs = FontDefinitions::default();
        for font in Self::iter() {
            font_defs.font_data.insert(
                String::from(font.id()),
                Arc::new(FontData::from_static(font.data())),
            );
        }
        font_defs.font_data.insert(
            String::from("icons"),
            Arc::new(FontData::from_static(ICONS)),
        );
        let proportional = font_defs
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap();
        proportional.insert(0, String::from(primary));
        proportional.push(String::from("icons"));
        ctx.set_fonts(font_defs);
    }

    pub fn install(ctx: &egui::Context) {
        Self::register(ctx, Self::FiraSans.id());
    }

    pub fn set(&self, ctx: &egui::Context) {
        Self::register(ctx, self.id());
    }
}

impl Display for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::DMSans => "DM Sans",
                Self::FiraSans => "Fira Sans",
                Self::Inter => "Inter",
                Self::Nunito => "Nunito",
                Self::Ubuntu => "Ubuntu",
            }
        )
    }
}
