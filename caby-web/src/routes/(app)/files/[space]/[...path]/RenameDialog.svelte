<script lang="ts">
	import Dialog from '$lib/Dialog.svelte';
	import { page } from '$app/state';
	import { PutEntryType } from '$lib/files/api';

	let { dialog = $bindable(), onListChange }: { dialog: HTMLDialogElement; onListChange: any } =
		$props();
	let value = $state();
	const path = $derived(page.params.path!);

	const tryRename = async () => {
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

	// fix this, was copy/pasted
	let renameEntry = $state({
		srcName: '',
		srcPath: '',
		dstName: ''
	});

	const renameEntryDialog = (entry: FileEntry | DirEntry) => {
		renameEntry.srcName = entry.name;
		renameEntry.srcPath = entry.path;
		renameEntry.dstName = entry.name;

		let dialog: HTMLDialogElement | null = document.querySelector('#rename-modal');
		dialog!.showModal();
	};
</script>

<Dialog bind:dialog title="Rename">
	{#snippet content()}
		<section>
			<form class="fx fx--col">
				<input id="input-folder-name" type="text" placeholder="Folder Name" bind:value />
				<button class="button" onclick={() => tryRename()}>Create</button>
			</form>
		</section>
	{/snippet}
</Dialog>
