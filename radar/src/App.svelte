<script lang="ts">
    import { onMount } from "svelte";
    import type { Data } from "./lib/data";
    import Radar from "./lib/Radar.svelte";

    let ws: WebSocket | null = null;
    let url: string | null = null;
    let uuid: string | null = null;
    let error: string | null = null;
    let data: Data | null = $state(null);
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    let reconnectAttempts = 0;
    let stopped = false;

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
        reconnectAttempts = 0;
        if (uuid !== null) {
            socket.send(uuid);
        }
    }

    function wsClose(socket: WebSocket, event: CloseEvent) {
        console.error(`websocket closed: ${event.code}`);
        if (ws === socket) {
            ws = null;
            scheduleReconnect();
        }
    }

    function wsError(socket: WebSocket) {
        console.error("websocket error");
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

<main>
    <Radar {data} />
</main>
