<script lang="ts">
	import type { Space } from '$lib/space';

	let { current_space, spaces }: { current_space: Space | undefined; spaces: Space[] } = $props();

	let popover: HTMLDivElement;
</script>

<button popovertarget="space-selector-menu" class="fx button spaces-button">
	<span class="fx-grow">{current_space?.display ?? current_space?.name ?? ''}</span>
	<iconify-icon icon="lucide:chevron-left" class="caret"></iconify-icon>
</button>

<div bind:this={popover} id="space-selector-menu" popover>
	{#each spaces as space}
		<a class="button" href="/files/{space.name}" onclick={() => popover.hidePopover()}
			>{space.display}</a
		>
	{/each}
</div>

<style lang="scss">
	.spaces-button {
		margin: 1rem;
		anchor-name: --spaces-button;
		align-items: center;

		.caret {
			transition: transform 0.2s;
		}
	}

	.spaces-button:has(+ #space-selector-menu:popover-open) .caret {
		transform: rotate(-90deg);
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
		box-shadow: var(--box-shadow-0);

		border: 0;
		// border-radius: 5px;
		background: var(--clr-background-2);

		&:popover-open {
			display: flex;
			flex-direction: column;
		}

		> .button {
			border-radius: 0;
			text-decoration: none;

			&:first-of-type {
				border-radius: 5px 5px 0 0;
			}

			&:last-of-type {
				border-radius: 0 0 5px 5px;
			}
		}
	}
</style>
