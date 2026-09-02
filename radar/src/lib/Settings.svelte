<script lang="ts">
    type Props = {
        open?: boolean;
    };

    let { open = false }: Props = $props();
</script>

{#if open}
    <div id="settings"></div>
{/if}
