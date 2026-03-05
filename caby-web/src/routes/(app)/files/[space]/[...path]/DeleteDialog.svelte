<script lang="ts">
	import { deleteFiles } from '$lib/api/api_files';
	import Dialog from '$lib/Dialog.svelte';
	import type { Entry } from '$lib/files/entry';
	import { client } from '$lib/stores/client.svelte';

	let {
		dialog = $bindable(),
		space,
		onListChange,
		entries
	}: {
		dialog: HTMLDialogElement;
		space: string;
		onListChange: any;
		entries: Array<Entry>;
	} = $props();

	const tryDelete = async () => {
		entries.forEach((e) => (e.is_processing = true));
		const resp = await deleteFiles(client, space, entries, true);
		if (resp.status != 'success') {
			console.error(`could not delete files: ${resp.message}`);
			return;
		}
		await onListChange();
		dialog.close();
	};
</script>

<Dialog bind:dialog title="Confirm Delete">
	{#snippet content()}
		<section>
			<div>
				{#if entries.length > 1}
					<p>Are you sure you want to delete the {entries.length} selected items?</p>
				{:else}
					<p>Are you sure you want to delete '{entries[0]?.name}'?</p>
				{/if}
			</div>
			<div class="actions fx">
				<button class="button" onclick={() => dialog.close()}>Cancel</button>
				<button class="button primary" onclick={() => tryDelete()} autofocus>Delete</button>
			</div>
		</section>
	{/snippet}
</Dialog>

<style lang="scss">
	section {
		padding: 1rem;
	}

	.actions {
		margin-top: 1rem;
		gap: 0.5rem;
		justify-content: end;

		> button {
			&.delete {
				color: var(--clr-background);
				background: linear-gradient(320deg, var(--clr-primary), var(--clr-accent));
			}
		}
	}
</style>
