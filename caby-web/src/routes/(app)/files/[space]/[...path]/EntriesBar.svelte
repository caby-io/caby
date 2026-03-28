<script lang="ts">
	import type { Entry } from '$lib/files/entry';
	import EntriesBreadcrumbs from './EntriesBreadcrumbs.svelte';
	import 'iconify-icon';

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
				<iconify-icon icon="lucide:file-x"></iconify-icon>
			</button>
			<button
				class="action selected fx fx--cc border-0 box-shadow-0-card"
				onclick={handleDeleteSelected}
				title="Delete selected"
			>
				<iconify-icon icon="lucide:trash-2"></iconify-icon>
			</button>
		{/if}
		<button class="action fx fx--cc border-0 box-shadow-0-card">
			<iconify-icon icon="ci:info"></iconify-icon>
		</button>
		<button
			class="action add fx fx--cc border-0 box-shadow-0-card"
			onclick={() => add_content_dialog.showModal()}
			title="Add/Upload content"
		>
			<iconify-icon icon="lets-icons:add-duotone"></iconify-icon>
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
			background: var(--clr-background-2);
			border-radius: 4px;
			height: 1rem;
			width: 1rem;
			padding: 0.5rem;

			&.selected {
				color: var(--clr-background-1);
				background: linear-gradient(320deg, var(--clr-primary), var(--clr-accent));
				opacity: 0.7;
				// border-color: var(--clr-accent);
			}

			&.add {
				font-size: 1.8rem;
				color: var(--clr-accent);
				// background: linear-gradient(320deg, var(--clr-primary), var(--clr-accent));
			}
		}
	}
</style>
