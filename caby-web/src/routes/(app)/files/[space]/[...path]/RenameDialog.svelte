<script lang="ts">
	import Dialog from '$lib/Dialog.svelte';
	import { page } from '$app/state';
	import { PutEntryType } from '$lib/files/api';
	import { moveFiles, type Move } from '$lib/api/api_files';
	import { client } from '$lib/stores/client.svelte';
	import type { Entry } from '$lib/files/entry';
	import { join } from '$lib/fs';

	let {
		space,
		target_rename_entry,
		dialog = $bindable(),
		onListChange
	}: {
		space: string;
		target_rename_entry?: Entry;
		dialog: HTMLDialogElement;
		onListChange: any;
	} = $props();

	let previous_name = $derived.by(() => {
		if (!target_rename_entry) {
			return '';
		}
		return target_rename_entry.path.substring(target_rename_entry.path.lastIndexOf('/') + 1);
	});

	let name: string = $state('');
	let target_path = $derived.by(() => {
		let root_path =
			target_rename_entry?.path.substring(0, target_rename_entry!.path.lastIndexOf('/')) || '';
		return join(root_path, name);
	});

	$effect(() => {
		name =
			target_rename_entry?.path.substring(target_rename_entry?.path.lastIndexOf('/') + 1) || '';
	});

	// todo: handle overwrites
	const tryRename = async () => {
		const move: Move = [target_rename_entry!.path, target_path];
		let resp = await moveFiles(client, space, [move]);
		if (resp.status != 'success') {
			console.error(`could not move files: ${resp.message}`);
			return;
		}

		await onListChange();
		dialog.close();
		name = '';
	};
</script>

<Dialog bind:dialog title="Rename">
	{#snippet content()}
		<section>
			<form
				class="fx fx--col"
				onsubmit={(e) => {
					e.preventDefault();
					tryRename();
				}}
			>
				<label for="input-folder-name">Renaming '{previous_name}'</label>
				<input id="input-folder-name" type="text" placeholder="Folder Name" bind:value={name} />
			</form>
			<div class="actions fx">
				<button class="button" onclick={() => dialog.close()}>Cancel</button>
				<button class="button primary" onclick={() => tryRename()} autofocus>Rename</button>
			</div>
		</section>
	{/snippet}
</Dialog>

<style lang="scss">
	section {
		padding: 1rem;
	}

	label {
		font-size: 0.8rem;
		margin-bottom: 0.5rem;
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
