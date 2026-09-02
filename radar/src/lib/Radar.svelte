<script lang="ts">
    import type { Data } from "./data";
    import EntityMarker from "./EntityMarker.svelte";
    import { MAP_DATA } from "./map_data";
    import PlayerCard from "./PlayerCard.svelte";
    import PlayerMarker from "./PlayerMarker.svelte";
    import type { RadarSettings } from "./settings";

    type Props = {
        data: Data | null;
        settings: RadarSettings;
    };

    const { data, settings }: Props = $props();
    const map = $derived(MAP_DATA[data?.map_name ?? ""]);

    let scale = $state(1);
    let x = $state(0);
    let y = $state(0);

    let allPlayers = $derived([
        ...(data?.players ?? []),
        ...(data?.friendlies ?? []),
        ...(data?.local_player ? [data.local_player] : []),
    ]);
    let terrorists = $derived(
        allPlayers.filter((player) => player.team === "T"),
    );
    let counterTerrorists = $derived(
        allPlayers.filter((player) => player.team === "CT"),
    );

    let dragging = $state(false);
    let startX = 0;
    let startY = 0;
    let startPanX = 0;
    let startPanY = 0;

    const MIN_ZOOM = 1;
    const MAX_ZOOM = 5;

    function onWheel(event: WheelEvent) {
        event.preventDefault();

        const rect = (event.currentTarget as Element).getBoundingClientRect();

        const mouseX = event.clientX - rect.left - rect.width / 2;
        const mouseY = event.clientY - rect.top - rect.height / 2;

        const oldScale = scale;
        const newScale = Math.min(
            MAX_ZOOM,
            Math.max(MIN_ZOOM, scale * (event.deltaY < 0 ? 1.1 : 0.9)),
        );

        if (newScale === oldScale) return;

        x = mouseX - (mouseX - x) * (newScale / oldScale);
        y = mouseY - (mouseY - y) * (newScale / oldScale);

        scale = newScale;
    }

    function onPointerDown(event: PointerEvent) {
        if (event.button !== 0) return;

        dragging = true;
        startX = event.clientX;
        startY = event.clientY;
        startPanX = x;
        startPanY = y;

        (event.currentTarget as Element).setPointerCapture(event.pointerId);
    }

    function onPointerMove(event: PointerEvent) {
        if (!dragging) return;

        x = startPanX + event.clientX - startX;
        y = startPanY + event.clientY - startY;
    }

    function onPointerUp() {
        dragging = false;
    }

    function resetView() {
        scale = 1;
        x = 0;
        y = 0;
    }
</script>

<div id="radar-container">
    <div
        id="radar"
        class="container"
        class:dragging
        onwheel={onWheel}
        onpointerdown={onPointerDown}
        onpointermove={onPointerMove}
        onpointerup={onPointerUp}
        onpointercancel={onPointerUp}
        role="region"
    >
        <div id="map" style:transform={`translate(${x}px, ${y}px) scale(${scale})`}>
            <div id="map-image" style:background-image={`url(/images/${data?.map_name}.png)`}></div>
            {#if map}
                {#each data?.entities ?? [] as entity}
                    <EntityMarker {entity} {map} size={settings.markerSize} />
                {/each}
            {/if}
            {#each data?.players as player}
                <PlayerMarker
                    {player}
                    friendly={false}
                    {map}
                    size={settings.markerSize}
                />
            {/each}
            {#each data?.friendlies as player}
                <PlayerMarker
                    {player}
                    friendly={true}
                    {map}
                    size={settings.markerSize}
                />
            {/each}
            {#if data?.local_player}
                <PlayerMarker
                    player={data?.local_player}
                    friendly={true}
                    {map}
                    size={settings.markerSize}
                />
            {/if}
        </div>
        <button
            id="reset"
            onclick={resetView}
            onpointerdown={(event) => event.stopPropagation()}
        >
            Reset
        </button>
    </div>

    <aside class="player-cards terrorists">
        {#each terrorists as player}
            <PlayerCard {player} />
        {/each}
    </aside>

    <aside class="player-cards counter-terrorists">
        {#each counterTerrorists as player}
            <PlayerCard {player} />
        {/each}
    </aside>
</div>

<style>
    #radar-container {
        position: relative;
        display: grid;
        grid-template-columns: 15rem minmax(0, 1fr) 15rem;
        width: min(100vw, calc(90vh + 31.5rem));
        height: auto;
        gap: 0.75rem;
        padding: 0 0.75rem;
        align-items: center;
    }

    #radar {
        grid-column: 2;
        width: 100%;
        aspect-ratio: 1 / 1;
        cursor: grab;
        position: relative;
        touch-action: none;
        overflow: hidden;
    }

    #radar.dragging {
        cursor: grabbing;
    }

    #map {
        position: absolute;
        transform-origin: center center;
        width: 100%;
        height: 100%;
    }

    #map-image {
        position: absolute;
        background-size: cover;
        width: 100%;
        height: 100%;
    }

    .player-cards {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        max-height: 90vh;
        overflow-y: auto;
        scrollbar-width: thin;
        align-self: center;
    }

    .player-cards.terrorists {
        grid-column: 1;
        grid-row: 1;
    }

    .player-cards.counter-terrorists {
        grid-column: 3;
        grid-row: 1;
    }

    #reset {
        cursor: pointer;
        position: absolute;
        top: 0.5rem;
        right: 0.5rem;
        z-index: 5;
    }

    @media (max-width: 1000px) {
        #radar-container {
            display: block;
            width: 100vw;
            padding: 0.5rem;
        }

        #radar {
            width: 100%;
            height: auto;
            aspect-ratio: 1;
        }

        .player-cards {
            display: none;
        }
    }
</style>
