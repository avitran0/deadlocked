use std::collections::HashMap;

use femtovg::{renderer::OpenGl, Canvas, ImageFlags, ImageId};

const FLAGS: ImageFlags = ImageFlags::empty();

macro_rules! icon {
    ($map:expr, $canvas:expr, $name:expr) => {
        $map.insert(
            $name,
            $canvas
                .load_image_mem(
                    include_bytes!(concat!("../resources/icons/png/", $name, ".png")),
                    FLAGS,
                )
                .unwrap(),
        );
    };
}

pub fn init(canvas: &mut Canvas<OpenGl>) -> HashMap<&'static str, ImageId> {
    let mut icons = HashMap::new();

    icon!(icons, canvas, "ak47");
    icon!(icons, canvas, "aug");
    icon!(icons, canvas, "awp");
    icon!(icons, canvas, "bayonet");
    icon!(icons, canvas, "bizon");
    icon!(icons, canvas, "c4");
    icon!(icons, canvas, "cz75a");
    icon!(icons, canvas, "deagle");
    icon!(icons, canvas, "decoy");
    icon!(icons, canvas, "elite");
    icon!(icons, canvas, "famas");
    icon!(icons, canvas, "firebomb");
    icon!(icons, canvas, "fiveseven");
    icon!(icons, canvas, "flashbang");
    icon!(icons, canvas, "g3sg1");
    icon!(icons, canvas, "galilar");
    icon!(icons, canvas, "glock");
    icon!(icons, canvas, "hegrenade");
    icon!(icons, canvas, "hkp2000");
    icon!(icons, canvas, "incgrenade");
    icon!(icons, canvas, "knife_bowie");
    icon!(icons, canvas, "knife_butterfly");
    icon!(icons, canvas, "knife_canis");
    icon!(icons, canvas, "knife_cord");
    icon!(icons, canvas, "knife_css");
    icon!(icons, canvas, "knife_falchion");
    icon!(icons, canvas, "knife_flip");
    icon!(icons, canvas, "knife_gut");
    icon!(icons, canvas, "knife_gypsy_jackknife");
    icon!(icons, canvas, "knife_kukri");
    icon!(icons, canvas, "knife_m9_bayonet");
    icon!(icons, canvas, "knife_outdoor");
    icon!(icons, canvas, "knife_push");
    icon!(icons, canvas, "knife_skeleton");
    icon!(icons, canvas, "knife_stiletto");
    icon!(icons, canvas, "knife_survival_bowie");
    icon!(icons, canvas, "knife_t");
    icon!(icons, canvas, "knife_tactical");
    icon!(icons, canvas, "knife_twinblade");
    icon!(icons, canvas, "knife_ursus");
    icon!(icons, canvas, "knife_widowmaker");
    icon!(icons, canvas, "knife");
    icon!(icons, canvas, "knifegg");
    icon!(icons, canvas, "m4a1_silencer_off");
    icon!(icons, canvas, "m4a1_silencer");
    icon!(icons, canvas, "m4a1");
    icon!(icons, canvas, "m249");
    icon!(icons, canvas, "mac10");
    icon!(icons, canvas, "mag7");
    icon!(icons, canvas, "molotov");
    icon!(icons, canvas, "mp5sd");
    icon!(icons, canvas, "mp7");
    icon!(icons, canvas, "mp9");
    icon!(icons, canvas, "negev");
    icon!(icons, canvas, "nova");
    icon!(icons, canvas, "p90");
    icon!(icons, canvas, "p250");
    icon!(icons, canvas, "p2000");
    icon!(icons, canvas, "revolver");
    icon!(icons, canvas, "sawedoff");
    icon!(icons, canvas, "scar20");
    icon!(icons, canvas, "sg556");
    icon!(icons, canvas, "smokegrenade");
    icon!(icons, canvas, "ssg08");
    icon!(icons, canvas, "taser");
    icon!(icons, canvas, "tec9");
    icon!(icons, canvas, "ump45");
    icon!(icons, canvas, "usp_silencer_off");
    icon!(icons, canvas, "usp_silencer");
    icon!(icons, canvas, "xm1014");

    icons
}
