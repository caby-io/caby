<script lang="ts">
	import type { Snippet } from 'svelte';
	import IconLucideChevronRight from '~icons/lucide/chevron-right';

	let { title, children }: { title: string; children: Snippet } = $props();
</script>

<details class="sidebar-section" open>
	<summary class="fx fx--ac">
		<span class="title fx-grow">{title}</span>
		<IconLucideChevronRight class="caret" />
	</summary>
	<div class="body">
		{@render children()}
	</div>
</details>

<style lang="scss">
	.sidebar-section {
		margin: 0.5rem 0.75rem;

		> summary {
			list-style: none;
			cursor: pointer;
			padding: 0.25rem 0.5rem;
			font-size: 0.75rem;
			font-weight: bold;
			letter-spacing: 0.08em;
			text-transform: uppercase;
			color: var(--clr-text-2);

			&::-webkit-details-marker {
				display: none;
			}

			:global(.caret) {
				transition: transform 0.2s;
			}
		}

		&[open] > summary :global(.caret) {
			transform: rotate(90deg);
		}
	}
</style>
