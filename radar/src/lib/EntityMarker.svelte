<script lang="ts">
    import type { EntityInfo, Vec2, Vec3 } from "./data";
    import { worldToRadar, type MapData } from "./map_data";
    import WeaponIcon from "./WeaponIcon.svelte";

    type Props = {
        entity: EntityInfo;
        map: MapData;
        size: number;
    };

    let { entity, map, size }: Props = $props();

    function convexHull(points: Vec2[]): Vec2[] {
        if (points.length < 3) return points;

        const sorted = [...points].sort(([ax, ay], [bx, by]) => ax - bx || ay - by);
        const unique: Vec2[] = [];

        for (const point of sorted) {
            const previous = unique[unique.length - 1];
            if (!previous || point[0] !== previous[0] || point[1] !== previous[1]) {
                unique.push(point);
            }
        }

        if (unique.length < 3) return unique;

        const cross = (origin: Vec2, a: Vec2, b: Vec2) =>
            (a[0] - origin[0]) * (b[1] - origin[1]) - (a[1] - origin[1]) * (b[0] - origin[0]);
        const lower: Vec2[] = [];
        const upper: Vec2[] = [];

        for (const point of unique) {
            while (
                lower.length >= 2 &&
                cross(lower[lower.length - 2], lower[lower.length - 1], point) <= 0
            ) {
                lower.pop();
            }
            lower.push(point);
        }

        for (let index = unique.length - 1; index >= 0; index--) {
            const point = unique[index];
            while (
                upper.length >= 2 &&
                cross(upper[upper.length - 2], upper[upper.length - 1], point) <= 0
            ) {
                upper.pop();
            }
            upper.push(point);
        }

        lower.pop();
        upper.pop();
        return lower.concat(upper);
    }

    function positionOf(entity: EntityInfo): Vec3 {
        if ("Weapon" in entity) return entity.Weapon.position;
        if ("Inferno" in entity) return entity.Inferno.position;
        if ("Molotov" in entity) return entity.Molotov.position;
        if ("Smoke" in entity) return entity.Smoke.position;
        if ("Flashbang" in entity) return entity.Flashbang.position;
        if ("HeGrenade" in entity) return entity.HeGrenade.position;
        if ("Decoy" in entity) return entity.Decoy.position;
        return [0, 0, 0];
    }

    function iconOf(entity: EntityInfo): string | null {
        if ("Smoke" in entity) return "smoke";
        if ("Flashbang" in entity) return "flashbang";
        if ("HeGrenade" in entity) return "h_e";
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
        return "";
    }

    let position = $derived(worldToRadar(positionOf(entity), map));
    let icon = $derived(iconOf(entity));
    let label = $derived(labelOf(entity));
    let hull = $derived(
        "Inferno" in entity
            ? convexHull(entity.Inferno.hull.map((point) => worldToRadar(point, map)))
            : [],
    );
</script>

{#if hull.length > 0}
    <svg class="inferno-hull" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
        <polygon points={hull.map(([x, y]) => `${x},${y}`).join(" ")} />
    </svg>
{/if}

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
    {/if}
</div>

<style>
    .inferno-hull {
        position: absolute;
        inset: 0;
        width: 100%;
        height: 100%;
        overflow: visible;
        pointer-events: none;
        fill: rgba(240, 100, 100, 0.35);
        stroke: var(--color-red);
        stroke-width: 0.35;
        stroke-linejoin: round;
    }

    .inferno-hull {
        fill: rgba(240, 100, 100, 0.35);
        stroke: var(--color-red);
        stroke-width: 0.35;
        stroke-linejoin: round;
    }

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
</style>
