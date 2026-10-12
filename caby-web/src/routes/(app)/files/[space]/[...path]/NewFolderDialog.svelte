<script lang="ts">
	import Dialog from '$lib/Dialog.svelte';
	import { page } from '$app/state';
	import { putDirectory } from '$lib/api/api_files';
	import { client } from '$lib/stores/client.svelte';

	let { space, dialog = $bindable() }: { space: string; dialog: HTMLDialogElement } = $props();
	let value: string = $state('');
	const path = $derived(page.params.path!);

	const tryCreateDir = async () => {
		await putDirectory(client, space, path, value);
		dialog.close();
		value = '';
	};
</script>

<Dialog bind:dialog title="New Folder">
	{#snippet content()}
		<section>
			<form class="fx fx--col">
				<input
					id="input-folder-name"
					type="text"
					placeholder="Folder Name"
					autocomplete="off"
					bind:value
				/>
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
