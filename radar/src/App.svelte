<script lang="ts">
    import { onMount } from "svelte";
    import type { Data } from "./lib/data";
    import Radar from "./lib/Radar.svelte";

    let ws: WebSocket | null = null;
    let url: string | null = null;
    let uuid: string | null = null;
    let error: string | null = null;
    let data: Data | null = $state(null);

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

        ws = new WebSocket(`ws://${url}/client`);
        ws.onopen = wsOpen;
        ws.onclose = wsClose;
        ws.onerror = wsError;
        ws.onmessage = wsMessage;
    });

    function wsOpen(event: Event) {
        if (uuid !== null) {
            ws?.send(uuid);
        }
    }

    function wsClose(event: CloseEvent) {
        console.error(`websocket closed: ${event.code}`);
    }

    function wsError(event: Event) {
        console.error("websocket error");
    }

    function wsMessage(event: MessageEvent) {
        data = JSON.parse(event.data);
    }
</script>

<main>
    <Radar {data} />
</main>
