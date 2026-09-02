<script lang="ts">
    import { type PlayerData } from "./data";

    type Props = {
        player: PlayerData;
    };

    let { player }: Props = $props();
</script>

<div>
    {player.name}
</div>

<style>
</style>
