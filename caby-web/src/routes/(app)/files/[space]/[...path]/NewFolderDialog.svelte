<script lang="ts">
	import Dialog from '$lib/Dialog.svelte';
	import { page } from '$app/state';
	import { PutEntryType } from '$lib/files/api';
	import { putEntry } from '$lib/api/api_files';
	import { client } from '$lib/stores/client.svelte';

	let {
		space,
		dialog = $bindable(),
		onListChange
	}: { space: string; dialog: HTMLDialogElement; onListChange: any } = $props();
	let value: string = $state('');
	const path = $derived(page.params.path!);

	const tryCreateDir = async () => {
		const resp = await putEntry(client, space, path, PutEntryType.DIRECTORY, value);
		console.log(resp);

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
