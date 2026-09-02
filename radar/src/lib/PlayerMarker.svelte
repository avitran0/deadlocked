<script lang="ts">
    import { playerColor } from "./color";
    import type { PlayerData } from "./data";
    import { worldToRadar, type MapData } from "./map_data";
    import Marker from "./Marker.svelte";
    import WeaponIcon from "./WeaponIcon.svelte";

    type Props = {
        player: PlayerData;
        friendly: boolean;
        map: MapData;
    };

    const { player, friendly, map }: Props = $props();
    let position = $derived(worldToRadar(player.position, map));
</script>

<div class="marker" style:left={`${position[0]}%`} style:top={`${position[1]}%`}>
    <Marker rotation={player.rotation} color={friendly ? playerColor(player) : "#f06464"} />
    <WeaponIcon icon={player.weapon} />
</div>

<style>
    .marker {
        position: absolute;
        width: 2.5%;
        height: 2.5%;
        transform: translate(-50%, -50%);
    }

    :global(.marker > svg) {
        position: absolute;
        inset: 0;
        width: 100%;
        height: 100%;
    }

    :global(.marker > img) {
        position: absolute;
        top: 100%;
        left: 50%;
        width: 65%;
        height: auto;
        transform: translateX(-50%);
    }
</style>
