<script lang="ts">
    import type { EntityInfo, Vec2, Vec3 } from "./data";
    import { worldToRadar, type MapData } from "./map_data";

    type Props = {
        entities: EntityInfo[];
        map: MapData;
        mapName: string;
    };

    let { entities, map, mapName }: Props = $props();
    let trails = $state(new Map<number, Vec2[]>());
    let moving = $state(new Set<number>());
    let lastPositions = $state(new Map<number, Vec3>());
    let trailMapName = "";

    function entityId(entity: EntityInfo): number | null {
        if ("Weapon" in entity) return null;
        if ("Inferno" in entity) return entity.Inferno.entity;
        if ("Molotov" in entity) return entity.Molotov.entity;
        if ("Smoke" in entity) return entity.Smoke.entity;
        if ("Flashbang" in entity) return entity.Flashbang.entity;
        if ("HeGrenade" in entity) return entity.HeGrenade.entity;
        if ("Decoy" in entity) return entity.Decoy.entity;
        return null;
    }

    function positionOf(entity: EntityInfo): [number, number, number] | null {
        if ("Weapon" in entity) return null;
        if ("Inferno" in entity) return entity.Inferno.position;
        if ("Molotov" in entity) return entity.Molotov.position;
        if ("Smoke" in entity) return entity.Smoke.position;
        if ("Flashbang" in entity) return entity.Flashbang.position;
        if ("HeGrenade" in entity) return entity.HeGrenade.position;
        if ("Decoy" in entity) return entity.Decoy.position;
        return null;
    }

    function colorOf(entity: EntityInfo): string {
        if ("Smoke" in entity) return "#d3d3d3";
        if ("Molotov" in entity) {
            return entity.Molotov.is_incendiary ? "#f08c5a" : "#f06464";
        }
        if ("Flashbang" in entity) return "#ffffff";
        if ("HeGrenade" in entity) return "#b0b0b0";
        if ("Decoy" in entity) return "#b478f0";
        return "#f06464";
    }

    function trailOf(entity: EntityInfo): Vec2[] {
        const id = entityId(entity);
        return id === null ? [] : (trails.get(id) ?? []);
    }

    function isMoving(entity: EntityInfo): boolean {
        const id = entityId(entity);
        return id !== null && moving.has(id);
    }

    $effect(() => {
        if (trailMapName !== mapName) {
            trails.clear();
            moving.clear();
            lastPositions.clear();
            trailMapName = mapName;
        }

        const currentIds = new Set<number>();
        for (const entity of entities) {
            const id = entityId(entity);
            const worldPosition = positionOf(entity);
            if (id === null || worldPosition === null) continue;

            currentIds.add(id);
            const position = worldToRadar(worldPosition, map);
            const trail = trails.get(id);
            const previous = trail?.[trail.length - 1];
            const lastPosition = lastPositions.get(id);
            // Use the same one-world-unit movement threshold as the cheat,
            // rather than comparing rounded radar percentages.
            const hasMoved =
                !lastPosition ||
                Math.hypot(
                    worldPosition[0] - lastPosition[0],
                    worldPosition[1] - lastPosition[1],
                    worldPosition[2] - lastPosition[2],
                ) >= 1;
            lastPositions.set(id, worldPosition);

            if (hasMoved) {
                moving.add(id);
                trails.set(id, [...(trail ?? []), position].slice(-600));
            } else {
                moving.delete(id);
                trails.delete(id);
            }
        }

        for (const id of trails.keys()) {
            if (!currentIds.has(id)) {
                trails.delete(id);
                lastPositions.delete(id);
            }
        }
        for (const id of moving) {
            if (!currentIds.has(id)) moving.delete(id);
        }
    });

    let smokeRadius = $derived((144 / map.scale / 1024) * 100);
</script>

{#each entities as entity}
    {#if trailOf(entity).length > 1}
        <svg class="entity-trail" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
            <polyline
                points={trailOf(entity).map(([x, y]) => `${x},${y}`).join(" ")}
                style:stroke={colorOf(entity)}
            />
        </svg>
    {/if}
    {#if "Smoke" in entity && !isMoving(entity)}
        <svg class="smoke-radius" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
            <circle
                cx={worldToRadar(entity.Smoke.position, map)[0]}
                cy={worldToRadar(entity.Smoke.position, map)[1]}
                r={smokeRadius}
            />
        </svg>
    {/if}
{/each}

<style>
    .entity-trail,
    .smoke-radius {
        position: absolute;
        inset: 0;
        width: 100%;
        height: 100%;
        overflow: visible;
        pointer-events: none;
    }

    .entity-trail {
        fill: none;
        stroke-width: 0.3;
        stroke-linecap: round;
        stroke-linejoin: round;
        opacity: 0.7;
    }

    .smoke-radius {
        fill: rgba(160, 160, 160, 0.12);
        stroke: rgba(190, 190, 190, 0.75);
        stroke-width: 0.35;
    }
</style>
