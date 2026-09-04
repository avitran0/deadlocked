export interface RadarSettings {
    markerSize: number;
    font: string;
    centerOnSelf: boolean;
    pipZoom: number;
}

export const DEFAULT_SETTINGS: RadarSettings = {
    markerSize: 3,
    font: "Lexend",
    centerOnSelf: false,
    pipZoom: 1,
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
            centerOnSelf:
                typeof stored?.centerOnSelf === "boolean"
                    ? stored.centerOnSelf
                    : DEFAULT_SETTINGS.centerOnSelf,
            pipZoom:
                typeof stored?.pipZoom === "number"
                    ? Math.min(4, Math.max(1, stored.pipZoom))
                    : DEFAULT_SETTINGS.pipZoom,
        };
    } catch {
        return { ...DEFAULT_SETTINGS };
    }
}

export function saveSettings(settings: RadarSettings): void {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
}
