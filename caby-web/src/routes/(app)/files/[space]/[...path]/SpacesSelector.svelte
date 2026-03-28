<script lang="ts">
	import type { Space } from '$lib/space';

	let {
		current_space,
		spaces
	}: { current_space: Space | undefined; spaces: Space[] } = $props();

	let popover: HTMLDivElement;
</script>

<button popovertarget="space-selector-menu" class="fx fx--cc button spaces-button">
	{current_space?.display ?? current_space?.name ?? ''}
</button>

<div bind:this={popover} id="space-selector-menu" popover>
	{#each spaces as space}
		<a class="fx fx--cc button" href="/files/{space.name}" onclick={() => popover.hidePopover()}
			>{space.display}</a
		>
	{/each}
</div>

<style lang="scss">
	.spaces-button {
		margin: 1rem;
		anchor-name: --spaces-button;
	}

	#space-selector-menu {
		position-anchor: --spaces-button;
		width: anchor-size(width);
		margin: 0;
		padding: 0;
		inset: auto;
		top: anchor(bottom);
		left: anchor(left);
		margin-block-start: 0.25rem;
		font-weight: normal;

		border: 0;
		border-radius: 5px;
		background: var(--clr-background-2);

		&:popover-open {
			display: flex;
			flex-direction: column;
		}

		> button {
			border-radius: 0;
		}
	}
</style>
