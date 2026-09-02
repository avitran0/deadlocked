<script lang="ts">
    import type { Data } from "./data";
    import { MAP_DATA } from "./map_data";
    import PlayerMarker from "./PlayerMarker.svelte";

    type Props = {
        data: Data | null;
    };

    const { data }: Props = $props();
    const map = $derived(MAP_DATA[data?.map_name ?? ""]);
</script>

<div id="radar" style:background-image={`url(/images/${data?.map_name}.png)`}>
    {data?.weapon}
    {#each data?.players as player}
        <PlayerMarker {player} {map} />
    {/each}
    {#each data?.friendlies as player}
        <PlayerMarker {player} {map} />
    {/each}
    {#if data?.local_player}
        <PlayerMarker player={data?.local_player} {map} />
    {/if}
</div>

<style>
    #radar {
        aspect-ratio: 1 / 1;
        background-size: cover;
        position: relative;
    }
</style>
