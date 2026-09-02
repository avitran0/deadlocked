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

<div id="marker" style:left={`${position[0]}%`} style:top={`${position[1]}%`}>
    <Marker rotation={player.rotation} color={friendly ? playerColor(player) : "#f06464"} />
    <WeaponIcon icon={player.weapon} />
</div>

<style>
    #marker {
        position: absolute;
        width: 2.5%;
        height: 2.5%;
        transform: translate(-50%, -50%);

        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
    }

    :global(#marker > svg:first-child) {
        position: absolute;
        width: 100%;
        height: 100%;
    }
</style>
