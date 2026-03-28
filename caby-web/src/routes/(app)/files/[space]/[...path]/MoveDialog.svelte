<script lang="ts">
	import { getFilesOverview, moveFiles, type Move } from '$lib/api/api_files';
	import Dialog from '$lib/Dialog.svelte';
	import type { Entry } from '$lib/files/entry';
	import EntriesOverviewSelect from '$lib/files/overview/EntriesOverviewSelect.svelte';
	import type { OverviewEntry } from '$lib/files/overview/overview_entry';
	import { join } from '$lib/fs';
	import { client } from '$lib/stores/client.svelte';

	let {
		dialog = $bindable(),
		space,
		path,
		onListChange,
		entries
	}: {
		dialog: HTMLDialogElement;
		space: string;
		path: string;
		onListChange: any;
		entries: Set<Entry>;
	} = $props();

	let overview_entries: Array<OverviewEntry> = $state([]);
	let selected_dir: OverviewEntry | undefined = $state();

	const expandPath = (entries: OverviewEntry[], segments: string[]) => {
		if (segments.length === 0) return;
		const entry = entries.find((e) => e.name === segments[0]);
		if (!entry) return;
		entry.is_expanded = true;
		expandPath(entry.children, segments.slice(1));
	};

	const fetchFilesOverview = async () => {
		const resp = await getFilesOverview(client, space, '', true);
		const entries = resp.data!.entries;
		if (path) {
			expandPath(entries, path.split('/'));
		}
		overview_entries = entries;
	};

	const handleSelect = (entry: OverviewEntry) => {
		if (selected_dir) selected_dir.is_selected = false;
		entry.is_selected = true;
		selected_dir = entry;
	};

	const tryMove = async () => {
		if (!selected_dir) return;
		const moves: Array<Move> = [...entries].map((e) => [e.path, join(selected_dir!.path, e.name)]);
		entries.forEach((e) => (e.is_processing = true));
		const resp = await moveFiles(client, space, moves, true);
		if (resp.status != 'success') {
			console.error(`could not move files: ${resp.message}`);
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
			<EntriesOverviewSelect {space} {overview_entries} onSelect={handleSelect} />
			<div class="actions fx">
				<button class="button" onclick={() => dialog.close()}>Cancel</button>
				<button class="button primary" onclick={() => tryMove()} disabled={!selected_dir}
					>Move</button
				>
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
