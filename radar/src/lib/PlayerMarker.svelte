<script lang="ts">
    import type { PlayerData } from "./data";
    import { worldToRadar, type MapData } from "./map_data";
    import Marker from "./Marker.svelte";

    type Props = { player: PlayerData; map: MapData };

    const { player, map }: Props = $props();
    let position = $derived(worldToRadar(player.position, map));
</script>

<div id="marker" style:left={`${position[0]}%`} style:top={`${position[1]}%`}>
    <Marker rotation={player.rotation} />
</div>

<style>
    #marker {
        position: absolute;
        width: 2%;
        height: 2%;
    }

    :global(#marker > svg) {
        width: 100%;
        height: 100%;
    }
</style>
