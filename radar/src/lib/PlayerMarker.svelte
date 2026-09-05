<script lang="ts">
    import { playerColor } from "./color";
    import type { PlayerData } from "./data";
    import { worldToRadar, type MapData } from "./map_data";
    import Marker from "./Marker.svelte";

    type Props = {
        player: PlayerData;
        friendly: boolean;
        map: MapData;
        size: number;
    };

    const { player, friendly, map, size }: Props = $props();
    let position = $derived(worldToRadar(player.position, map));
</script>

<div
    class="marker"
    style:left={`${position[0] - size / 2}%`}
    style:top={`${position[1] - size / 2}%`}
    style:width={`${size}%`}
    style:height={`${size}%`}
    style:transform={`rotate(${-player.rotation + 90}deg)`}
>
    <Marker color={friendly ? playerColor(player) : "#f06464"} />
    <p>{player.name}</p>
</div>

<style>
    .marker {
        position: absolute;
        container-type: size;
    }

    .marker p {
        position: absolute;
        top: 100%;
        left: 50%;
        margin: 0;
        transform: translateX(-50%);
        color: var(--color-text);
        font-size: clamp(0.65rem, 40cqw, 1.2rem);
        line-height: 1;
        text-align: center;
        text-shadow: 0 1px 2px var(--color-backdrop);
        white-space: nowrap;
        pointer-events: none;
    }

    :global(.marker > svg) {
        position: absolute;
        inset: 0;
        width: 100%;
        height: 100%;
    }
</style>
