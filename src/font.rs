use std::{fmt::Display, sync::Arc};

use egui::{FontData, FontDefinitions, FontFamily};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

const ICONS: &[u8] = include_bytes!("../resources/Icons.ttf");

#[derive(Debug, Clone, Default, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum Font {
    DMSans,
    #[default]
    FiraSans,
    Inter,
    JetBrainsMono,
    Lato,
    Nunito,
    OpenSans,
    Roboto,
    Ubuntu,
}

impl Font {
    pub fn data(&self) -> &'static [u8] {
        match self {
            Self::DMSans => include_bytes!("../resources/DMSans.ttf"),
            Self::FiraSans => include_bytes!("../resources/FiraSans.ttf"),
            Self::Inter => include_bytes!("../resources/Inter.ttf"),
            Self::JetBrainsMono => include_bytes!("../resources/JetBrainsMono.ttf"),
            Self::Lato => include_bytes!("../resources/Lato.ttf"),
            Self::Nunito => include_bytes!("../resources/Nunito.ttf"),
            Self::OpenSans => include_bytes!("../resources/OpenSans.ttf"),
            Self::Roboto => include_bytes!("../resources/Roboto.ttf"),
            Self::Ubuntu => include_bytes!("../resources/Ubuntu.ttf"),
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            Self::DMSans => "dm_sans",
            Self::FiraSans => "fira_sans",
            Self::Inter => "inter",
            Self::JetBrainsMono => "jetbrains_mono",
            Self::Lato => "lato",
            Self::Nunito => "nunito",
            Self::OpenSans => "open_sans",
            Self::Roboto => "roboto",
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
        writeln!(
            f,
            "{}",
            match self {
                Self::DMSans => "DM Sans",
                Self::FiraSans => "Fira Sans",
                Self::Inter => "Inter",
                Self::JetBrainsMono => "JetBrains Mono",
                Self::Lato => "Lato",
                Self::Nunito => "Nunito",
                Self::OpenSans => "Open Sans",
                Self::Roboto => "Roboto",
                Self::Ubuntu => "Ubuntu",
            }
        )
    }
}
