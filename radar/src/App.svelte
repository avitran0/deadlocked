<script lang="ts">
    import { onMount } from "svelte";
    import type { Data } from "./lib/data";
    import Radar from "./lib/Radar.svelte";
    import Settings from "./lib/Settings.svelte";
    import { loadSettings, saveSettings, type RadarSettings } from "./lib/settings";

    let ws: WebSocket | null = null;
    let url: string | null = null;
    let uuid: string | null = null;
    let error: string | null = null;
    let data: Data | null = $state(null);
    let settings = $state<RadarSettings>(loadSettings());
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    let reconnectAttempts = 0;
    let stopped = false;
    let connectionStatus = $state<"connecting" | "connected" | "disconnected">("connecting");

    $effect(() => {
        const font = settings.font.trim();
        if (!font) return;

        const fontFace = new FontFace(font, `url("/fonts/${encodeURIComponent(font)}.woff2")`);
        fontFace.load().then((loadedFont) => document.fonts.add(loadedFont));
    });

    $effect(() => {
        saveSettings(settings);
    });

    onMount(() => {
        const query = new URLSearchParams(window.location.search);
        url = query.get("url");
        uuid = query.get("game");

        if (url === null) {
            error = "No URL specified";
            return;
        }

        if (uuid === null) {
            error = "No Game UUID specified";
            return;
        }

        connect();

        return () => {
            stopped = true;
            if (reconnectTimer !== null) {
                clearTimeout(reconnectTimer);
            }
            ws?.close();
        };
    });

    function connect() {
        if (stopped || url === null || uuid === null) return;

        connectionStatus = "connecting";
        const socket = new WebSocket(`ws://${url}/client`);
        ws = socket;
        socket.onopen = () => wsOpen(socket);
        socket.onclose = (event) => wsClose(socket, event);
        socket.onerror = () => wsError(socket);
        socket.onmessage = wsMessage;
    }

    function scheduleReconnect() {
        if (stopped || reconnectTimer !== null) return;

        const delay = Math.min(10_000, 1_000 * 2 ** reconnectAttempts);
        reconnectAttempts += 1;
        reconnectTimer = setTimeout(() => {
            reconnectTimer = null;
            connect();
        }, delay);
    }

    function wsOpen(socket: WebSocket) {
        connectionStatus = "connected";
        reconnectAttempts = 0;
        if (uuid !== null) {
            socket.send(uuid);
        }
    }

    function wsClose(socket: WebSocket, event: CloseEvent) {
        console.error(`websocket closed: ${event.code}`);
        connectionStatus = "disconnected";
        if (ws === socket) {
            ws = null;
            scheduleReconnect();
        }
    }

    function wsError(socket: WebSocket) {
        console.error("websocket error");
        connectionStatus = "disconnected";
        socket.close();
    }

    function wsMessage(event: MessageEvent) {
        try {
            data = JSON.parse(event.data);
        } catch (error) {
            console.error("failed to parse websocket message", error);
        }
    }
</script>

<main
    style:--font-family={`"${settings.font}", sans-serif`}
    style:font-family={`"${settings.font}", sans-serif`}
>
    <Settings bind:settings />
    <div id="connection-status" class={connectionStatus}>
        {connectionStatus === "connecting"
            ? "Connecting..."
            : connectionStatus === "connected"
              ? "Connected"
              : "Disconnected - retrying..."}
    </div>
    <Radar {data} {settings} />
</main>

<style>
    #connection-status {
        position: fixed;
        top: 0.75rem;
        left: 50%;
        z-index: 100;
        padding: 0.4rem 0.6rem;
        border: var(--border);
        border-radius: var(--border-radius);
        background: var(--color-base);
        box-shadow: var(--box-shadow);
        transform: translateX(-50%);
        font-size: var(--font-size-small);
    }

    #connection-status.connecting {
        color: var(--color-yellow);
    }

    #connection-status.connected {
        color: var(--color-green);
    }

    #connection-status.disconnected {
        color: var(--color-red);
    }
</style>
