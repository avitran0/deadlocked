<script lang="ts">
    type Props = {
        icon: string;
    };

    const { icon }: Props = $props();
</script>

<img src={`/icons/${icon}.svg`} alt={icon} />

<style>
    img {
        height: 50%;
    }
</style>
