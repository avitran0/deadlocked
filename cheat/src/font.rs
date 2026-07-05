use std::{fmt::Display, sync::Arc};

use egui::{FontData, FontDefinitions, FontFamily};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, Default, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum Font {
    Lexend,
    DMSans,
    #[default]
    FiraSans,
    Nunito,
    Rubik,
    Ubuntu,
}

impl Font {
    pub fn data(&self) -> &'static [u8] {
        match self {
            Self::Lexend => include_bytes!("../resources/Lexend.ttf"),
            Self::DMSans => include_bytes!("../resources/DMSans.ttf"),
            Self::FiraSans => include_bytes!("../resources/FiraSans.ttf"),
            Self::Nunito => include_bytes!("../resources/Nunito.ttf"),
            Self::Rubik => include_bytes!("../resources/Rubik.ttf"),
            Self::Ubuntu => include_bytes!("../resources/Ubuntu.ttf"),
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            Self::Lexend => "lexend",
            Self::DMSans => "dm_sans",
            Self::FiraSans => "fira_sans",
            Self::Nunito => "nunito",
            Self::Rubik => "rubik",
            Self::Ubuntu => "ubuntu",
        }
    }

    pub fn install(ctx: &egui::Context) {
        let mut font_defs = FontDefinitions::default();
        for font in Self::iter() {
            font_defs.font_data.insert(
                String::from(font.id()),
                Arc::new(FontData::from_static(font.data())),
            );
        }
        font_defs
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, String::from(Self::Lexend.id()));
        ctx.set_fonts(font_defs);
    }

    pub fn set(&self, ctx: &egui::Context) {
        let mut font_defs = FontDefinitions::default();
        for font in Font::iter() {
            font_defs.font_data.insert(
                String::from(font.id()),
                Arc::new(FontData::from_static(font.data())),
            );
        }
        font_defs
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, String::from(self.id()));
        ctx.set_fonts(font_defs);
    }
}

impl Display for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}",
            match self {
                Self::Lexend => "Lexend",
                Self::DMSans => "DM Sans",
                Self::FiraSans => "Fira Sans",
                Self::Nunito => "Nunito",
                Self::Rubik => "Rubik",
                Self::Ubuntu => "Ubuntu",
            }
        )
    }
}
