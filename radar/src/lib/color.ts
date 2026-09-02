import { Color, type PlayerData, type Team } from "./data";

export const COLORS = {
    backdrop: "#18181c",
    base: "#1e1e23",
    highlight: "#32323c",
    subtext: "#8c8c8c",
    text: "#ffffff",
    red: "#f06464",
    orange: "#f08c5a",
    yellow: "#f0c878",
    green: "#a0f082",
    teal: "#50c8c8",
    blue: "#6496f0",
    purple: "#b478f0",
} as const;

export function playerColor(player: PlayerData): string {
    switch (player.color) {
        case Color.Yellow:
            return COLORS.yellow;
        case Color.Purple:
            return COLORS.purple;
        case Color.Green:
            return COLORS.green;
        case Color.Blue:
            return COLORS.blue;
        case Color.Orange:
            return COLORS.orange;
        default:
            return COLORS.subtext;
    }
}

export function teamColor(team: Team): string {
    switch (team) {
        case "T":
            return COLORS.orange;
        case "CT":
            return COLORS.blue;
        default:
            return COLORS.subtext;
    }
}

export function healthColor(health: number, maxHealth: number): string {
    const maximum = Math.max(1, maxHealth);
    const percent = Math.max(0, Math.min(1, health / maximum));

    if (percent <= 0.5) {
        const yellowPercent = percent * 200;
        return `color-mix(in srgb, ${COLORS.red} ${100 - yellowPercent}%, ${COLORS.yellow})`;
    }

    const greenPercent = (percent - 0.5) * 200;
    return `color-mix(in srgb, ${COLORS.yellow} ${100 - greenPercent}%, ${COLORS.green})`;
}
