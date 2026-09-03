<script lang="ts">
    import type { RadarSettings } from "./settings";

    type Props = {
        settings: RadarSettings;
    };

    let { settings = $bindable() }: Props = $props();
</script>

<details id="settings" class="container">
    <summary>Settings</summary>

    <label>
        Font
        <input
            type="text"
            bind:value={settings.font}
            placeholder="Google Font name"
        />
    </label>

    <label>
        Marker size
        <input
            type="range"
            min="1"
            max="5"
            step="0.1"
            bind:value={settings.markerSize}
        />
        <output>{settings.markerSize.toFixed(1)}%</output>
    </label>
</details>

<style>
    #settings {
        position: fixed;
        top: 0.75rem;
        left: 0.75rem;
        z-index: 100;
        padding: 0.4rem 0.6rem;
    }

    summary {
        cursor: pointer;
    }

    label {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        padding-top: 0.6rem;
        white-space: nowrap;
    }

    input[type="text"] {
        width: 12rem;
    }

    input[type="range"] {
        width: 8rem;
    }

    output {
        min-width: 2.5rem;
        text-align: right;
    }
</style>
