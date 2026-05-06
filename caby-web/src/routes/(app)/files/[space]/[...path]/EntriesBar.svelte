<script lang="ts">
	import type { Entry } from '$lib/files/entry';
	import EntriesBreadcrumbs from './EntriesBreadcrumbs.svelte';
	import IconLucideFileX from '~icons/lucide/file-x';
	import IconLucideTrash2 from '~icons/lucide/trash-2';
	import IconCiInfo from '~icons/ci/info';
	import IconLetsIconsAddDuotone from '~icons/lets-icons/add-duotone';

	let {
		selected_entries,
		add_content_dialog,
		space,
		handleDeleteSelected,
		handleDeselect
	}: {
		selected_entries: Set<Entry>;
		add_content_dialog: HTMLDialogElement;
		space: string;
		handleDeleteSelected: any;
		handleDeselect: any;
	} = $props();
</script>

<header class="fx fx--ac">
	<EntriesBreadcrumbs {space} />
	<div class="actions fx fx--ac">
		{#if selected_entries.size > 0}
			<span>{selected_entries.size} selected</span>
			<button
				class="action fx fx--cc border-0 box-shadow-0-card"
				onclick={handleDeselect}
				title="Deselect"
			>
				<IconLucideFileX />
			</button>
			<button
				class="action selected fx fx--cc border-0 box-shadow-0-card"
				onclick={handleDeleteSelected}
				title="Delete selected"
			>
				<IconLucideTrash2 />
			</button>
		{/if}
		<button class="action fx fx--cc border-0 box-shadow-0-card">
			<IconCiInfo />
		</button>
		<button
			class="action add fx fx--cc border-0 box-shadow-0-card"
			onclick={() => add_content_dialog.showModal()}
			title="Add/Upload content"
		>
			<IconLetsIconsAddDuotone />
		</button>
	</div>
</header>

<style lang="scss">
	header {
		// height: 2.6rem;
		background-color: var(--clr-background-1);
		border-bottom: 1px solid var(--clr-border);
		padding: 0 1rem;
	}

	.actions {
		margin-left: auto;
		gap: 0.5rem;
		padding: 0.5rem 0;
		font-size: 1.5rem;

		> span {
			font-size: 1rem;
			color: var(--clr-text-2);
		}

		.action {
			cursor: pointer;
			box-sizing: border-box;
			background: var(--clr-background-2);
			border-radius: 4px;
			height: 2rem;
			width: 2rem;
			padding: 0.25rem;

			&.selected {
				color: var(--clr-background-1);
				background: linear-gradient(320deg, var(--clr-primary), var(--clr-accent));
				opacity: 0.7;
				// border-color: var(--clr-accent);
			}

			&.add {
				color: var(--clr-accent);
				padding: 0.1rem;
				// background: linear-gradient(320deg, var(--clr-primary), var(--clr-accent));
			}
		}
	}
</style>
