export interface RadarSettings {
    markerSize: number;
}

export const DEFAULT_SETTINGS: RadarSettings = {
    markerSize: 3,
};

const STORAGE_KEY = "settings";

export function loadSettings(): RadarSettings {
    if (typeof localStorage === "undefined") {
        return { ...DEFAULT_SETTINGS };
    }

    try {
        const stored = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}");

        return {
            ...DEFAULT_SETTINGS,
            ...(typeof stored === "object" && stored !== null ? stored : {}),
            markerSize:
                typeof stored?.markerSize === "number"
                    ? Math.min(5, Math.max(1, stored.markerSize))
                    : DEFAULT_SETTINGS.markerSize,
        };
    } catch {
        return { ...DEFAULT_SETTINGS };
    }
}

export function saveSettings(settings: RadarSettings): void {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
}
