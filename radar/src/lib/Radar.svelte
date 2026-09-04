<script lang="ts">
    import { onMount } from "svelte";
    import type { Data, GrenadeInfo, PlayerData, Vec3 } from "./data";
    import EntityMarker from "./EntityMarker.svelte";
    import { MAP_DATA, worldToRadar } from "./map_data";
    import GrenadeTrails from "./GrenadeTrails.svelte";
    import PlayerCard from "./PlayerCard.svelte";
    import PlayerMarker from "./PlayerMarker.svelte";
    import type { RadarSettings } from "./settings";
    import { COLORS } from "./color";

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
    let isPip = $state(false);
    let pipCanvas: HTMLCanvasElement;
    let pipVideo: HTMLVideoElement;
    let streamStarted = false;
    const mapImages: Record<string, HTMLImageElement> = {};

    // same path used by Marker.svelte, in its 20x20 viewBox coordinate space
    const MARKER_PATH = new Path2D("M6 7L10 2L14 7A5 5 270 1 1 6 7");

    function loadMapImage(name: string): HTMLImageElement | null {
        if (!name) return null;
        if (!mapImages[name]) {
            const img = new Image();
            img.src = `/images/${name}.png`;
            mapImages[name] = img;
        }
        const img = mapImages[name];
        return img.complete && img.naturalWidth > 0 ? img : null;
    }

    function entityPosition(entity: (typeof activeEntities)[number]): Vec3 | null {
        if ("Weapon" in entity) return entity.Weapon.position;
        if ("Inferno" in entity) return entity.Inferno.position;
        if ("Molotov" in entity) return entity.Molotov.position;
        const grenade = (entity as { [key: string]: GrenadeInfo })[
            Object.keys(entity)[0]
        ];
        return grenade?.position ?? null;
    }

    function drawFrame() {
        if (!pipCanvas) return;
        const ctx = pipCanvas.getContext("2d");
        if (!ctx) return;

        const size = pipCanvas.width;
        ctx.setTransform(1, 0, 0, 1, 0, 0);
        ctx.shadowBlur = 0;
        ctx.fillStyle = COLORS.backdrop;
        ctx.fillRect(0, 0, size, size);

        if (!map) {
            ctx.fillStyle = COLORS.text;
            ctx.font = "16px sans-serif";
            ctx.textAlign = "center";
            ctx.fillText("No map loaded", size / 2, size / 2);
            return;
        }

        const local = data?.local_player;
        const centering = settings.centerOnSelf && local && data?.in_game;
        const [refX, refY] = centering ? worldToRadar(local.position, map) : [50, 50];
        const viewRotationDeg = centering ? local.rotation - 90 : 0;

        // returns the final absolute canvas pixel position for a world point,
        // already centered on the reference player and rotated to match the
        // view (mirrors what the CSS rotate() on #map does to every marker
        // nested inside it in the normal HTML radar)
        const zoom = settings.pipZoom;

        function project(world: Vec3): [number, number] {
            const [px, py] = worldToRadar(world, map);
            const ex = ((px - refX) / 100) * size;
            const ey = ((py - refY) / 100) * size;
            const rad = (viewRotationDeg * Math.PI) / 180;
            const rx = ex * Math.cos(rad) - ey * Math.sin(rad);
            const ry = ex * Math.sin(rad) + ey * Math.cos(rad);
            return [size / 2 + rx * zoom, size / 2 + ry * zoom];
        }

        // the map image always covers percent-space (0,0)-(100,100); position
        // its top-left corner so that (refX, refY) lands on the canvas center,
        // then rotate/zoom the whole frame around that same center
        const img = loadMapImage(data?.map_name ?? "");
        ctx.save();
        ctx.translate(size / 2, size / 2);
        ctx.rotate((viewRotationDeg * Math.PI) / 180);
        ctx.scale(zoom, zoom);
        if (img) {
            ctx.drawImage(img, -(refX / 100) * size, -(refY / 100) * size, size, size);
        }
        ctx.restore();

        for (const entity of activeEntities) {
            const position = entityPosition(entity);
            if (!position) continue;
            const [sx, sy] = project(position);
            ctx.fillStyle = COLORS.subtext;
            ctx.beginPath();
            ctx.arc(sx, sy, 3, 0, Math.PI * 2);
            ctx.fill();
        }

        for (const player of data?.players ?? []) {
            drawPlayerMarker(ctx, player, COLORS.red, size, project, viewRotationDeg);
        }
        const isLocalPlayer = (player: PlayerData) =>
            local !== undefined && player.steam_id === local.steam_id;

        for (const player of data?.friendlies ?? []) {
            drawPlayerMarker(
                ctx,
                player,
                COLORS.blue,
                size,
                project,
                viewRotationDeg,
                isLocalPlayer(player),
            );
        }
        if (local && !(data?.friendlies ?? []).some(isLocalPlayer)) {
            drawPlayerMarker(ctx, local, COLORS.blue, size, project, viewRotationDeg, true);
        }

        if (data?.bomb.planted) {
            const [sx, sy] = project(data.bomb.position);
            const defusing = data.bomb.being_defused;
            const bombColor = defusing ? COLORS.green : COLORS.red;
            const pulsePeriod = defusing ? 600 : 1100;
            const pulseT = (Date.now() % pulsePeriod) / pulsePeriod;

            ctx.beginPath();
            ctx.arc(sx, sy, 8 + pulseT * 20, 0, Math.PI * 2);
            ctx.fillStyle = `color-mix(in srgb, ${bombColor} ${50 * (1 - pulseT)}%, transparent)`;
            ctx.fill();

            ctx.beginPath();
            ctx.arc(sx, sy, 8, 0, Math.PI * 2);
            ctx.fillStyle = bombColor;
            ctx.strokeStyle = COLORS.backdrop;
            ctx.lineWidth = 2;
            ctx.shadowColor = bombColor;
            ctx.shadowBlur = 10;
            ctx.fill();
            ctx.stroke();
            ctx.shadowBlur = 0;

            ctx.fillStyle = COLORS.text;
            ctx.font = "bold 8px sans-serif";
            ctx.textAlign = "center";
            ctx.textBaseline = "middle";
            ctx.fillText("C4", sx, sy);
            ctx.textBaseline = "alphabetic";

            ctx.font = "bold 13px sans-serif";
            ctx.fillStyle = bombColor;
            const label = defusing
                ? `defusing ${data.bomb.defuse_remain_time.toFixed(1)}s`
                : data.bomb.timer.toFixed(1);
            ctx.fillText(label, sx, sy - 20);
        }
    }

    function drawPlayerMarker(
        ctx: CanvasRenderingContext2D,
        player: PlayerData,
        color: string,
        size: number,
        project: (world: Vec3) => [number, number],
        viewRotationDeg: number,
        isLocal = false,
    ) {
        if (!map) return;
        const [cx, cy] = project(player.position);
        const markerPx = (settings.markerSize / 100) * size;

        const margin = markerPx + 10;
        const half = size / 2 - margin;
        const dx = cx - size / 2;
        const dy = cy - size / 2;
        if (Math.abs(dx) > half || Math.abs(dy) > half) {
            drawEdgeIndicator(ctx, size, dx, dy, color);
            return;
        }

        if (isLocal) {
            // pulsing halo so your own marker is instantly findable
            const pulse = 3 + Math.sin(Date.now() / 200) * 2;
            ctx.beginPath();
            ctx.arc(cx, cy, markerPx + 8 + pulse, 0, Math.PI * 2);
            ctx.strokeStyle = COLORS.yellow;
            ctx.lineWidth = 2.5;
            ctx.stroke();
            ctx.shadowColor = COLORS.yellow;
            ctx.shadowBlur = 12;
        }

        // total on-screen rotation = the view rotation (map spin) composed
        // with the marker's own heading, same as the nested CSS transforms
        // used by Marker.svelte inside the rotated #map div
        const iconRotationDeg = viewRotationDeg + (-player.rotation + 90);

        ctx.save();
        ctx.translate(cx, cy);
        ctx.rotate((iconRotationDeg * Math.PI) / 180);
        const scale = markerPx / 20;
        ctx.scale(scale, scale);
        ctx.translate(-10, -10);
        ctx.fillStyle = color;
        ctx.strokeStyle = `color-mix(in srgb, ${color} 70%, black)`;
        ctx.lineWidth = 1;
        ctx.fill(MARKER_PATH);
        ctx.stroke(MARKER_PATH);
        ctx.restore();

        ctx.shadowBlur = 0;
    }

    // player is outside the visible canvas - draw a small arrow clamped to
    // the edge, pointing toward their real direction (matches CS2's own
    // radar behavior for teammates who are off the zoomed-in view)
    function drawEdgeIndicator(
        ctx: CanvasRenderingContext2D,
        size: number,
        dx: number,
        dy: number,
        color: string,
    ) {
        const center = size / 2;
        const inset = 16;
        const bound = center - inset;
        const scale = bound / Math.max(Math.abs(dx), Math.abs(dy), 1e-6);
        const ex = center + dx * scale;
        const ey = center + dy * scale;
        const angle = Math.atan2(dy, dx);

        ctx.save();
        ctx.translate(ex, ey);
        ctx.rotate(angle + Math.PI / 2);
        ctx.beginPath();
        ctx.moveTo(0, -7);
        ctx.lineTo(6, 6);
        ctx.lineTo(-6, 6);
        ctx.closePath();
        ctx.fillStyle = color;
        ctx.strokeStyle = COLORS.backdrop;
        ctx.lineWidth = 1.5;
        ctx.fill();
        ctx.stroke();
        ctx.restore();
    }

    $effect(() => {
        void data;
        drawFrame();
    });

    onMount(() => {
        // set the stream up and start playback ahead of time, so the click
        // handler below can call requestPictureInPicture() as the very first
        // await with nothing before it - some browsers silently refuse PiP
        // if it isn't close enough to the original click/user-gesture
        pipVideo.srcObject = pipCanvas.captureStream(30);
        pipVideo.play().catch((err) => console.error("radar pip video play failed:", err));
        streamStarted = true;

        pipVideo.addEventListener("enterpictureinpicture", () => {
            isPip = true;
        });
        pipVideo.addEventListener("leavepictureinpicture", () => {
            isPip = false;
        });
    });

    async function togglePip() {
        try {
            if (document.pictureInPictureElement) {
                await document.exitPictureInPicture();
                return;
            }

            if (!document.pictureInPictureEnabled) {
                alert(
                    "Picture-in-Picture is disabled in this browser (check browser settings/site permissions).",
                );
                return;
            }

            if (!streamStarted) {
                pipVideo.srcObject = pipCanvas.captureStream(30);
                await pipVideo.play();
                streamStarted = true;
            }

            await pipVideo.requestPictureInPicture();
        } catch (err) {
            console.error("Picture-in-Picture failed:", err);
            alert(`Couldn't open Picture-in-Picture: ${err instanceof Error ? err.message : String(err)}`);
        }
    }

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
    let effectiveFollowed = $derived(
        settings.centerOnSelf ? (data?.local_player ?? undefined) : followedPlayer,
    );
    let mapRotation = $derived(
        effectiveFollowed ? effectiveFollowed.rotation - 90 : 0,
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
        if (effectiveFollowed) centerPlayer(effectiveFollowed);
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
            {#if map}
                <div
                    id="map-image"
                    style:background-image={`url(/images/${data?.map_name}.png)`}
                ></div>
                <GrenadeTrails
                    entities={activeEntities}
                    {map}
                    mapName={data?.map_name ?? ""}
                />
                {#each activeEntities as entity}
                    <EntityMarker {entity} {map} size={settings.markerSize} />
                {/each}
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
                        player={data.local_player}
                        friendly={true}
                        {map}
                        size={settings.markerSize}
                    />
                {/if}
                {#if data?.bomb.planted}
                    {@const bombPos = worldToRadar(data.bomb.position, map)}
                    <div
                        class="bomb-marker"
                        class:defusing={data.bomb.being_defused}
                        style:left={`${bombPos[0]}%`}
                        style:top={`${bombPos[1]}%`}
                    >
                        <div class="bomb-pulse"></div>
                        <div class="bomb-icon">C4</div>
                        <div class="bomb-timer">
                            {data.bomb.being_defused
                                ? `defusing ${data.bomb.defuse_remain_time.toFixed(1)}s`
                                : data.bomb.timer.toFixed(1)}
                        </div>
                    </div>
                {/if}
            {:else}
                <div id="no-map">No map loaded</div>
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
        <button
            id="pip-toggle"
            onclick={togglePip}
            onpointerdown={(event) => event.stopPropagation()}
        >
            {isPip ? "Exit picture-in-picture" : "Pop out"}
        </button>
    </div>

    <canvas
        bind:this={pipCanvas}
        width="480"
        height="480"
        style="position: fixed; top: -9999px; left: -9999px;"
    ></canvas>
    <!-- svelte-ignore a11y_media_has_caption -->
    <video
        bind:this={pipVideo}
        muted
        playsinline
        style="position: fixed; top: -9999px; left: -9999px; width: 1px; height: 1px;"
    ></video>

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

    #no-map {
        position: absolute;
        inset: 0;
        display: grid;
        place-items: center;
        color: var(--color-text);
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
    #unfollow,
    #pip-toggle {
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

    #pip-toggle {
        left: 50%;
        transform: translateX(-50%);
    }

    .bomb-marker {
        position: absolute;
        transform: translate(-50%, -50%);
        display: flex;
        flex-direction: column;
        align-items: center;
        z-index: 4;
        pointer-events: none;
    }

    .bomb-pulse {
        position: absolute;
        top: 50%;
        left: 50%;
        width: 1.6rem;
        height: 1.6rem;
        margin: -0.8rem 0 0 -0.8rem;
        border-radius: 50%;
        background: color-mix(in srgb, var(--color-red) 50%, transparent);
        animation: bomb-pulse 1.1s ease-out infinite;
    }

    .bomb-marker.defusing .bomb-pulse {
        background: color-mix(in srgb, var(--color-green) 50%, transparent);
        animation-duration: 0.6s;
    }

    .bomb-icon {
        position: relative;
        width: 1.1rem;
        height: 1.1rem;
        border-radius: 50%;
        background: var(--color-red);
        color: var(--color-text);
        font-size: 0.5rem;
        font-weight: bold;
        display: grid;
        place-items: center;
        border: 2px solid var(--color-backdrop);
        box-shadow: 0 0 6px 1px var(--color-red);
    }

    .bomb-marker.defusing .bomb-icon {
        background: var(--color-green);
        box-shadow: 0 0 6px 1px var(--color-green);
    }

    .bomb-timer {
        margin-top: 0.15rem;
        font-size: 0.65rem;
        font-weight: bold;
        color: var(--color-text);
        text-shadow: 0 1px 2px var(--color-backdrop);
        white-space: nowrap;
    }

    @keyframes bomb-pulse {
        0% {
            transform: scale(1);
            opacity: 0.9;
        }
        100% {
            transform: scale(2.4);
            opacity: 0;
        }
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
