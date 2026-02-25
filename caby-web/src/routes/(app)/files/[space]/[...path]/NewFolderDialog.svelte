<script lang="ts">
	import Dialog from '$lib/Dialog.svelte';
	import { page } from '$app/state';
	import { PutEntryType } from '$lib/files/api';

	let { dialog = $bindable(), onListChange }: { dialog: HTMLDialogElement; onListChange: any } =
		$props();
	let value = $state();
	const path = $derived(page.params.path!);

	const tryCreateDir = async () => {
		const response = await fetch('http://localhost:8080/v0/files/' + path, {
			method: 'put',
			headers: {
				Accept: 'application/json',
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				entry_type: PutEntryType.DIRECTORY,
				name: value
			})
		});

		onListChange();
		dialog.close();
		value = '';
	};
</script>

<Dialog bind:dialog title="New Folder">
	{#snippet content()}
		<section>
			<form class="fx fx--col">
				<input id="input-folder-name" type="text" placeholder="Folder Name" bind:value />
				<button class="button primary" onclick={() => tryCreateDir()}>Create</button>
			</form>
		</section>
	{/snippet}
</Dialog>

<style lang="scss">
	section {
		padding: 1rem;
	}

	form {
		gap: 1rem;

		.button {
			align-self: flex-end;
		}
	}
</style>
