<script lang="ts">
    import type { Data, PlayerData } from "./data";
    import EntityMarker from "./EntityMarker.svelte";
    import { MAP_DATA, worldToRadar } from "./map_data";
    import GrenadeTrails from "./GrenadeTrails.svelte";
    import PlayerCard from "./PlayerCard.svelte";
    import PlayerMarker from "./PlayerMarker.svelte";
    import type { RadarSettings } from "./settings";

    type Props = {
        data: Data | null;
        settings: RadarSettings;
    };

    const { data, settings }: Props = $props();
    const map = $derived(MAP_DATA[data?.map_name ?? ""]);

    function playerKey(player: PlayerData): string {
        const identity = `${player.steam_id}:${player.name}`;
        let hash = 2166136261;

        for (let index = 0; index < identity.length; index++) {
            hash ^= identity.charCodeAt(index);
            hash = Math.imul(hash, 16777619) >>> 0;
        }

        return hash.toString(16);
    }

    let scale = $state(1);
    let x = $state(0);
    let y = $state(0);
    let radarElement: HTMLDivElement;
    let followedPlayerKey = $state<string | null>(null);

    let allPlayers = $derived([
        ...(data?.players ?? []),
        ...(data?.friendlies ?? []),
        ...(data?.local_player ? [data.local_player] : []),
    ]);
    let activeEntities = $derived(
        (data?.entities ?? []).filter((entity) => {
            if ("Inferno" in entity) return entity.Inferno.hull.length > 0;
            if ("Weapon" in entity) {
                return ![
                    "flashbang",
                    "h_e",
                    "smoke",
                    "molotov",
                    "incendiary",
                    "decoy",
                ].includes(entity.Weapon.weapon);
            }
            return true;
        }),
    );
    let terrorists = $derived(
        allPlayers.filter((player) => player.team === "T"),
    );
    let counterTerrorists = $derived(
        allPlayers.filter((player) => player.team === "CT"),
    );
    let followedPlayer = $derived(
        allPlayers.find((player) => playerKey(player) === followedPlayerKey),
    );
    let mapRotation = $derived(
        followedPlayer ? followedPlayer.rotation - 90 : 0,
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

    function centerPlayer(player: PlayerData) {
        if (!radarElement || !map) return;

        const position = worldToRadar(player.position, map);
        const width = radarElement.clientWidth;
        const height = radarElement.clientHeight;

        const dx = (position[0] / 100 - 0.5) * width;
        const dy = (position[1] / 100 - 0.5) * height;
        const angle = ((player.rotation - 90) * Math.PI) / 180;
        const cos = Math.cos(angle);
        const sin = Math.sin(angle);
        const rotatedX = dx * cos - dy * sin;
        const rotatedY = dx * sin + dy * cos;

        x = -rotatedX * scale;
        y = -rotatedY * scale;
    }

    function followPlayer(player: PlayerData) {
        if (followedPlayerKey === playerKey(player)) {
            unfollowPlayer();
            return;
        }

        followedPlayerKey = playerKey(player);
        centerPlayer(player);
    }

    function unfollowPlayer() {
        followedPlayerKey = null;
        resetView();
    }

    $effect(() => {
        if (followedPlayer) centerPlayer(followedPlayer);
    });

    function resetView() {
        followedPlayerKey = null;
        scale = 1;
        x = 0;
        y = 0;
    }
</script>

<div id="radar-container">
    <div
        id="radar"
        class="container"
        bind:this={radarElement}
        class:dragging
        onwheel={onWheel}
        onpointerdown={onPointerDown}
        onpointermove={onPointerMove}
        onpointerup={onPointerUp}
        onpointercancel={onPointerUp}
        role="region"
    >
        <div
            id="map"
            style:transform={`translate(${x}px, ${y}px) scale(${scale}) rotate(${mapRotation}deg)`}
        >
            <div id="map-image" style:background-image={`url(/images/${data?.map_name}.png)`}></div>
            {#if map}
                <GrenadeTrails
                    entities={activeEntities}
                    {map}
                    mapName={data?.map_name ?? ""}
                />
                {#each activeEntities as entity}
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
        {#if followedPlayer}
            <button
                id="unfollow"
                onclick={unfollowPlayer}
                onpointerdown={(event) => event.stopPropagation()}
            >
                Unfollow
            </button>
        {/if}
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
            <PlayerCard
                {player}
                onclick={followPlayer}
                followed={followedPlayerKey === playerKey(player)}
            />
        {/each}
    </aside>

    <aside class="player-cards counter-terrorists">
        {#each counterTerrorists as player}
            <PlayerCard
                {player}
                onclick={followPlayer}
                followed={followedPlayerKey === playerKey(player)}
            />
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

    #reset,
    #unfollow {
        cursor: pointer;
        position: absolute;
        top: 0.5rem;
        z-index: 5;
    }

    #reset {
        right: 0.5rem;
    }

    #unfollow {
        left: 0.5rem;
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
