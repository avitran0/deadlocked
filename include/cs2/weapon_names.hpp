#pragma once

#include <string>
#include <unordered_map>

const std::unordered_map<std::string, const char *> weapon_names = {
    // Pistols
    {"cz75a", "CZ75-Auto"},
    {"deagle", "Desert Eagle"},
    {"elite", "Dual Berettas"},
    {"fiveseven", "Five-SeveN"},
    {"glock", "Glock-18"},
    {"hkp2000", "P2000"},
    {"p250", "P250"},
    {"revolver", "R8 Revolver"},
    {"tec9", "Tec-9"},
    {"usp_silencer", "USP-S"},
    {"usp_silencer_off", "USP"},

    // SMGs
    {"bizon", "PP-Bizon"},
    {"mac10", "MAC-10"},
    {"mp5sd", "MP5-SD"},
    {"mp7", "MP7"},
    {"mp9", "MP9"},
    {"p90", "P90"},
    {"ump45", "UMP-45"},

    // Heavy
    {"m249", "M249"},
    {"negev", "Negev"},

    // Shotguns
    {"mag7", "MAG-7"},
    {"nova", "Nova"},
    {"sawedoff", "Sawed-Off"},
    {"xm1014", "XM1014"},

    // Rifles
    {"ak47", "AK-47"},
    {"aug", "AUG"},
    {"famas", "FAMAS"},
    {"galilar", "Galil AR"},
    {"m4a1_silencer", "M4A1-S"},
    {"m4a1_silencer_off", "M4A4"},
    {"m4a1", "M4A1"},
    {"sg556", "SG 553"},

    // Snipers
    {"awp", "AWP"},
    {"g3sg1", "G3SG1"},
    {"scar20", "SCAR-20"},
    {"ssg08", "SSG 08"},

    // Utility
    {"taser", "Zeus x27"}};
