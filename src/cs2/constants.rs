pub struct Constants;
impl Constants {
    pub const PROCESS_NAME: &str = "cs2";
    pub const CLIENT_LIB: &str = "libclient.so";
    pub const ENGINE_LIB: &str = "libengine2.so";
    pub const TIER0_LIB: &str = "libtier0.so";
    pub const INPUT_LIB: &str = "libinputsystem.so";
    pub const SDL_LIB: &str = "libSDL3.so.0";
    pub const MATCHMAKING_LIB: &str = "libmatchmaking.so";

    pub const TEAM_T: u8 = 2;
    pub const TEAM_CT: u8 = 3;

    pub const WEAPON_UNKNOWN: &str = "unknown";
}
