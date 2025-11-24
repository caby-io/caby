<script lang="ts">
	import { join } from '$lib/fs';
	import Self from './OverviewEntry.svelte';

	let { entry = $bindable() } = $props();
</script>

<section class="entry-branch">
	<div class="entry fx" class:expanded={entry.is_expanded}>
		<div class="control fx fx--cc" onclick={() => (entry.is_expanded = !entry.is_expanded)}>
			<iconify-icon icon="lucide:chevron-right"></iconify-icon>
		</div>
		<a class="fx" href={`/${join(`files`, entry.path)}`}>
			<div class="icon">📁</div>
			<div class="name fx-grow">{entry.name}</div>
		</a>
	</div>
	<section class="children" class:expanded={entry.is_expanded}>
		{#each entry.children as _, i}
			<Self bind:entry={entry.children[i]} />
		{/each}
	</section>
</section>

<style lang="scss">
	.entry {
		margin: 0 1rem;
		padding: 0.5rem;
		cursor: pointer;

		&:hover {
			background: var(--clr-background-2);
			border-radius: 5px;
		}

		.control {
			font-size: 1.2rem;
			margin-right: 0.5rem;
			transition: transform 0.2s;
		}

		.name {
			margin: 0 0.5rem;
		}

		&.expanded {
			.control {
				transform: rotate(90deg);
			}
		}
	}

	.children {
		margin-left: 0.75rem;
		height: 0;
		overflow: hidden;
		visibility: hidden;
		// interpolate-size: allow-keywords;
		transition: height 0.2s;

		&.expanded {
			height: auto;
			visibility: visible;
		}
	}
</style>
