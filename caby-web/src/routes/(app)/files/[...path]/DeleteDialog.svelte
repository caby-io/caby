<script lang="ts">
	import Dialog from '$lib/Dialog.svelte';
	import { page } from '$app/state';
	import type { Entry } from '$lib/files/entry';

	let {
		dialog = $bindable(),
		onListChange,
		entries
	}: { dialog: HTMLDialogElement; onListChange: any; entries: Array<Entry> } = $props();

	const tryDelete = async () => {
		const response = await fetch('http://localhost:8080/v0/files/delete', {
			method: 'post',
			headers: {
				Accept: 'application/json',
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				entries: entries.map((e) => e.path),
				force: true
			})
		});

		onListChange();
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
				<button class="button delete" onclick={() => tryDelete()}>Delete</button>
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
				background: var(--clr-error);
			}
		}
	}
</style>
