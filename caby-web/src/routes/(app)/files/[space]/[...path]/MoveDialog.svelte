<script lang="ts">
	import { getFilesOverview, moveFiles } from '$lib/api/api_files';
	import Dialog from '$lib/Dialog.svelte';
	import type { Entry } from '$lib/files/entry';
	import EntriesOverview from '$lib/files/overview/EntriesOverviewNav.svelte';
	import EntriesOverviewSelect from '$lib/files/overview/EntriesOverviewSelect.svelte';
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
		entries: Set<Entry>;
	} = $props();

	let overview_entries: any = $state();

	const fetchFilesOverview = async () => {
		const resp = await getFilesOverview(client, space, '', true);
		overview_entries = resp.data!.entries;
	};

	const tryMove = async () => {
		entries.forEach((e) => (e.is_processing = true));
		const resp = await moveFiles(client, space, entries, true);
		if (resp.status != 'success') {
			console.error(`could not delete files: ${resp.message}`);
			return;
		}
		await onListChange();
		dialog.close();
	};

	$effect(() => {
		fetchFilesOverview();
	});
</script>

<Dialog bind:dialog title="Move {entries.size} items to:">
	{#snippet content()}
		<section>
			<EntriesOverviewSelect {space} {overview_entries} />
			<div class="actions fx">
				<button class="button" onclick={() => dialog.close()}>Cancel</button>
				<button class="button primary" onclick={() => tryMove()} autofocus>Move</button>
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
