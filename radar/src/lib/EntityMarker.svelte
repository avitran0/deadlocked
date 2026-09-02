<script lang="ts">
    import type { EntityInfo, Vec3 } from "./data";
    import { worldToRadar, type MapData } from "./map_data";
    import WeaponIcon from "./WeaponIcon.svelte";

    type Props = {
        entity: EntityInfo;
        map: MapData;
        size: number;
    };

    let { entity, map, size }: Props = $props();

    function positionOf(entity: EntityInfo): Vec3 {
        if ("Weapon" in entity) return entity.Weapon.position;
        if ("Inferno" in entity) return entity.Inferno.position;
        if ("Molotov" in entity) return entity.Molotov.position;
        if ("Smoke" in entity) return entity.Smoke.position;
        if ("Flashbang" in entity) return entity.Flashbang.position;
        if ("HeGrenade" in entity) return entity.HeGrenade.position;
        if ("Decoy" in entity) return entity.Decoy.position;
        return entity.Chicken.position;
    }

    function iconOf(entity: EntityInfo): string | null {
        if ("Smoke" in entity) return "smoke";
        if ("Flashbang" in entity) return "flashbang";
        if ("HeGrenade" in entity) return "hegrenade";
        if ("Decoy" in entity) return "decoy";
        if ("Molotov" in entity) return "molotov";
        if ("Inferno" in entity) return "inferno";
        return null;
    }

    function labelOf(entity: EntityInfo): string {
        if ("Smoke" in entity) return entity.Smoke.name;
        if ("Flashbang" in entity) return entity.Flashbang.name;
        if ("HeGrenade" in entity) return entity.HeGrenade.name;
        if ("Decoy" in entity) return entity.Decoy.name;
        if ("Molotov" in entity) return entity.Molotov.is_incendiary ? "Incendiary" : "Molotov";
        if ("Inferno" in entity) return "Inferno";
        if ("Chicken" in entity) return "Chicken";
        return "";
    }

    let position = $derived(worldToRadar(positionOf(entity), map));
    let icon = $derived(iconOf(entity));
    let label = $derived(labelOf(entity));
</script>

<div
    class="entity-marker"
    style:left={`${position[0]}%`}
    style:top={`${position[1]}%`}
    style:width={`${size}%`}
    style:height={`${size}%`}
>
    {#if "Weapon" in entity}
        <WeaponIcon icon={entity.Weapon.weapon} />
    {:else if icon !== null}
        <img src={`/icons/${icon}.svg`} alt={label} />
    {:else}
        <span class="chicken-marker" aria-label={label}>●</span>
    {/if}

    {#if label}
        <span class="label">{label}</span>
    {/if}
</div>

<style>
    .entity-marker {
        position: absolute;
        display: flex;
        align-items: center;
        justify-content: center;
        transform: translate(-50%, -50%);
        pointer-events: none;
        white-space: nowrap;
    }

    :global(.entity-marker > img),
    :global(.entity-marker > div) {
        width: 100%;
        height: 100%;
        object-fit: contain;
    }

    :global(.entity-marker > div img) {
        width: 100%;
        height: 100%;
        object-fit: contain;
    }

    .chicken-marker {
        color: var(--color-yellow);
        font-size: 1rem;
        line-height: 1;
    }

    .label {
        position: absolute;
        top: 100%;
        padding: 0.1rem 0.25rem;
        border-radius: var(--border-radius);
        background: var(--color-base);
        color: var(--color-text);
        font-size: var(--font-size-xsmall);
    }
</style>
