<script lang="ts">
    type Props = {
        color: string;
    };

    const { color }: Props = $props();
</script>

<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" width="20" height="20">
    <path
        d="M6 7L10 2L14 7A5 5 270 1 1 6 7"
        stroke-linejoin="round"
        fill={color}
        stroke={`color-mix(in srgb, ${color} 70%, black)`}
    />
</svg>
