import { Color, type PlayerData } from "./data";

export function playerColor(player: PlayerData): string {
    switch (player.color) {
        case Color.Yellow:
            return "#f0c878";
        case Color.Purple:
            return "#b478f0";
        case Color.Green:
            return "#a0f082";
        case Color.Blue:
            return "#6496f0";
        case Color.Orange:
            return "#f08c5a";
        default:
            return "#8c8c8c";
    }
}
