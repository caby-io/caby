<script lang="ts">
	import { join } from '$lib/fs';
	import { expanded_dirs } from '$lib/stores/overview.svelte';
	import type { OverviewEntry } from './overview_entry';
	import Self from './OverviewEntryNav.svelte';
	import IconLucideChevronRight from '~icons/lucide/chevron-right';
	import IconFlatColorIconsFolder from '~icons/flat-color-icons/folder';
	import IconFlatColorIconsOpenedFolder from '~icons/flat-color-icons/opened-folder';

	let {
		entry,
		href_base,
		current_path
	}: { entry: OverviewEntry; href_base: string; current_path: string } = $props();

	const key = $derived(join(href_base, entry.path));
	const expanded = $derived(expanded_dirs.has(key));
	const active = $derived(join(current_path) === join(entry.path));
	const has_children = $derived(entry.children.length > 0);

	const toggle = () => {
		if (expanded) expanded_dirs.delete(key);
		else expanded_dirs.add(key);
	};
</script>

<section class="entry-branch">
	<div class="entry fx fx--ac" class:active>
		<a class="fx fx--ac fx-grow" href={`/${key}`}>
			<span class="icon fx fx--ac">
				{#if expanded}
					<IconFlatColorIconsOpenedFolder />
				{:else}
					<IconFlatColorIconsFolder />
				{/if}
			</span>
			<span class="name fx-grow ellipsis">{entry.name}</span>
		</a>
		{#if has_children}
			<button
				class="control fx fx--cc"
				class:expanded
				aria-label={expanded ? 'Collapse folder' : 'Expand folder'}
				aria-expanded={expanded}
				onclick={toggle}
			>
				<IconLucideChevronRight />
			</button>
		{/if}
	</div>
	{#if has_children}
		<section class="children" class:expanded>
			{#each entry.children as child (child.path)}
				<Self entry={child} {href_base} {current_path} />
			{/each}
		</section>
	{/if}
</section>

<style lang="scss">
	.entry {
		border-radius: 5px;
		transition: background-color 0.15s ease;

		&:hover {
			background-color: var(--clr-background-2);
		}

		&.active {
			background-color: color-mix(in oklab, var(--clr-accent) 25%, transparent);

			.name {
				color: var(--clr-primary);
			}
		}

		> a {
			min-width: 0;
			padding: 0.4rem 0.5rem;
			color: var(--clr-text-1);
			text-decoration: none;
		}

		.name {
			margin: 0 0.5rem;
			transition: color 0.15s ease;
		}

		.control {
			padding: 0.25rem;
			margin-right: 0.25rem;
			cursor: pointer;
			transition: transform 0.2s;

			&.expanded {
				transform: rotate(90deg);
			}
		}
	}

	.children {
		margin-left: 1rem;
		padding-left: 0.25rem;
		border-left: 1px solid var(--clr-border);
		height: 0;
		overflow: hidden;
		visibility: hidden;
		transition: height 0.2s;

		&.expanded {
			height: auto;
			visibility: visible;
		}
	}
</style>
