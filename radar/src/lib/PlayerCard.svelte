<script lang="ts">
    import { healthColor, teamColor } from "./color";
    import { type PlayerData } from "./data";
    import WeaponIcon from "./WeaponIcon.svelte";

    type Props = {
        player: PlayerData;
        onclick?: (player: PlayerData) => void;
        followed?: boolean;
    };

    let { player, onclick, followed = false }: Props = $props();
    let color = $derived(teamColor(player.team));
    let healthColorValue = $derived(healthColor(player.health, player.max_health));
    let armorPercent = $derived(Math.max(0, Math.min(100, player.armor)));
    let healthPercent = $derived(
        player.max_health > 0
            ? Math.max(0, Math.min(100, (player.health / player.max_health) * 100))
            : 0,
    );
</script>

<button
    type="button"
    class="player-card container"
    class:followed
    style:border-color={color}
    aria-label={`Follow ${player.name || "Unknown player"}`}
    aria-pressed={followed}
    onclick={() => onclick?.(player)}
>
    <div class="heading">
        <strong>{player.name || "Unknown player"}</strong>
        <span class="money">${player.money.toLocaleString()}</span>
    </div>

    <div class="health-row">
        <span>HP {player.health}/{player.max_health}</span>
        <span>Armor {player.armor}</span>
    </div>
    <div class="health-bar">
        <div style:width={`${healthPercent}%`} style:background-color={healthColorValue}></div>
    </div>
    <div class="armor-bar">
        <div style:width={`${armorPercent}%`}></div>
    </div>

    <div class="weapon-row">
        <div class="weapon-icon">
            <WeaponIcon icon={player.weapon} />
        </div>
        <span>{player.ammo[0]}/{player.ammo[1]}</span>
    </div>

    <div class="statuses">
        {#if player.has_helmet}
            <img class="status-icon" src="/icons/helmet.svg" alt="Helmet" title="Helmet" />
        {/if}
        {#if player.has_defuser}
            <img class="status-icon" src="/icons/defuser.svg" alt="Defuser" title="Defuser" />
        {/if}
        {#if player.has_bomb}
            <img class="status-icon" src="/icons/c4.svg" alt="Bomb" title="Bomb" />
        {/if}
    </div>
</button>

<style>
    .player-card {
        display: block;
        width: 15rem;
        max-width: 100%;
        padding: 0.5rem 0.65rem;
        text-align: left;
        cursor: pointer;
        transition: var(--transition-linear);
    }

    .player-card:hover,
    .player-card:focus-visible,
    .player-card.followed {
        background: var(--color-highlight);
    }

    .player-card:focus-visible {
        outline: 2px solid var(--color-text);
        outline-offset: 2px;
    }

    .heading,
    .health-row,
    .weapon-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 0.5rem;
    }

    .heading {
        margin-bottom: 0.35rem;
    }

    .heading strong {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .heading span {
        font-size: var(--font-size-xsmall);
    }

    .health-row,
    .weapon-row,
    .statuses {
        color: var(--color-subtext);
        font-size: var(--font-size-small);
    }

    .health-bar,
    .armor-bar {
        height: 0.3rem;
        margin: 0.25rem 0 0.45rem;
        overflow: hidden;
        border-radius: var(--border-radius);
        background: var(--color-highlight);
    }

    .health-bar div,
    .armor-bar div {
        height: 100%;
        transition: var(--transition-linear);
    }

    .armor-bar div {
        background: var(--color-blue);
    }

    .weapon-row {
        color: var(--color-text);
    }

    .weapon-icon {
        width: 3rem;
        height: 1.4rem;
    }

    :global(.weapon-icon img) {
        width: 100%;
        height: 100%;
        object-fit: contain;
    }

    .statuses {
        display: flex;
        flex-wrap: wrap;
        gap: 0.35rem;
        margin-top: 0.4rem;
    }

    .status-icon {
        width: 1.1rem;
        height: 1.1rem;
        object-fit: contain;
        opacity: 0.85;
    }

    @media (max-width: 800px) {
        .player-card {
            width: min(15rem, 78vw);
            flex: 0 0 min(15rem, 78vw);
        }
    }
</style>
