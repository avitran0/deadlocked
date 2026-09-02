export interface RadarSettings {}

export const DEFAULT_SETTINGS: RadarSettings = {};

const STORAGE_KEY = "settings";

export function loadSettings(): RadarSettings {
    if (typeof localStorage === "undefined") {
        return { ...DEFAULT_SETTINGS };
    }

    try {
        const stored = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}");

        return typeof stored === "object" && stored !== null ? stored : {};
    } catch {
        return { ...DEFAULT_SETTINGS };
    }
}

export function saveSettings(settings: RadarSettings): void {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
}
