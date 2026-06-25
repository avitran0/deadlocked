<script lang="ts">
    import { onMount } from "svelte";
    import { type Data, defaultData } from "./lib/data";

    let canvas: HTMLCanvasElement;
    let errorDialog: HTMLDialogElement;

    let data: Data = defaultData();
    let uuid: string | null = null;
    let ws: WebSocket | null = null;

    onMount(() => {
        const ctx = canvas.getContext("2d")!;
        requestAnimationFrame(() => render(ctx));

        const query = new URLSearchParams(window.location.search);
        uuid = query.get("uuid");
        if (!uuid) {
            errorDialog.showModal();
        }

        const origin = window.location.origin;
        const url = new URL(origin);
        url.pathname = "client";
        ws = new WebSocket(url);
        ws.onopen = wsOpen;
        ws.onmessage = wsMessage;
        ws.onclose = wsClose;
    });

    function wsOpen() {
        console.info("opened websocket connection");
    }

    function wsMessage(event: MessageEvent) {
        if (typeof event.data !== "string") {
            return;
        }

        const msg = event.data;
        const json: Data = JSON.parse(msg);
    }

    function wsClose() {
        console.info("closed websocket connection");
    }

    function render(ctx: CanvasRenderingContext2D) {
        const dpr = window.devicePixelRatio;
        canvas.width = canvas.clientWidth * dpr;
        canvas.height = canvas.clientHeight * dpr;
        ctx.clearRect(0, 0, canvas.width, canvas.height);

        requestAnimationFrame(() => render(ctx));
    }
</script>

<div class="mapname">{data.map_name ?? "map_name"}</div>
<canvas bind:this={canvas}></canvas>
<dialog bind:this={errorDialog}>
    <h1>Game not available</h1>
</dialog>

<style>
    .mapname {
        position: absolute;
        top: 1rem;
        left: 1rem;
    }

    canvas {
        width: 100dvw;
        height: 100dvh;
        position: absolute;
        top: 0;
        left: 0;
        z-index: -5;
    }
</style>
