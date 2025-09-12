<script lang="ts">
	import Dialog from '$lib/Dialog.svelte';
	import { page } from '$app/stores';
	import { PutEntryType } from '$lib/files/api';

	let { dialog = $bindable(), onListChange }: { dialog: HTMLDialogElement; onListChange: any } =
		$props();
	let value = $state();

	const tryCreateDir = async () => {
		const response = await fetch('http://localhost:8080/v0/files/' + $page.params.path, {
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

		console.debug(response);

		onListChange();
		dialog.close();
		value = '';
	};
</script>

<Dialog bind:dialog title="New Folder">
	<section>
		<form class="fx fx--col">
			<input id="input-folder-name" type="text" placeholder="Folder Name" bind:value />
			<button class="button" onclick={() => tryCreateDir()}>Submit</button>
		</form>
	</section>
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
