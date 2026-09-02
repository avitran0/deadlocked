<script lang="ts">
    import type { Data } from "./data";
    import { MAP_DATA } from "./map_data";
    import PlayerCard from "./PlayerCard.svelte";
    import PlayerMarker from "./PlayerMarker.svelte";

    type Props = {
        data: Data | null;
    };

    const { data }: Props = $props();
    const map = $derived(MAP_DATA[data?.map_name ?? ""]);

    let scale = $state(1);
    let x = $state(0);
    let y = $state(0);

    let terrorists = $derived(data?.players.filter((player) => player.team === "T") ?? []);
    let counterTerrorists = $derived([
        ...(data?.friendlies ?? []),
        ...(data?.local_player ? [data.local_player] : []),
    ].filter((player) => player.team === "CT"));

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
            {#each data?.players as player}
                <PlayerMarker {player} friendly={false} {map} />
            {/each}
            {#each data?.friendlies as player}
                <PlayerMarker {player} friendly={true} {map} />
            {/each}
            {#if data?.local_player}
                <PlayerMarker player={data?.local_player} friendly={true} {map} />
            {/if}
        </div>
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

    <button id="reset" onclick={resetView}>Reset</button>
</div>

<style>
    #radar-container {
        position: relative;
        width: min(90vw, 90vh);
        height: min(90vw, 90vh);
    }

    #radar {
        aspect-ratio: 1 / 1;
        background-color: var(--color-backdrop);
        border: var(--border);
        border-radius: var(--border-radius);
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
        position: absolute;
        top: 0;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        max-height: 100%;
        overflow-y: auto;
        scrollbar-width: thin;
    }

    .player-cards.terrorists {
        right: calc(100% + 0.75rem);
    }

    .player-cards.counter-terrorists {
        left: calc(100% + 0.75rem);
    }

    #reset {
        cursor: pointer;
        position: absolute;
        top: 0.5rem;
        right: 0.5rem;
        z-index: 5;
    }
</style>
