<script module lang="ts">
	export const enum MoveOp {
		ADD_SRC,
		REM_SRC,
		SET_DST,
		REM_DST,
		EXEC
	}

	export type UploadRegistration = {
		id: string;
		chunk_size: number;
		token: string;
	};
</script>

<script lang="ts">
	import { page } from '$app/state';
	import * as fs from '$lib/fs';

	import 'iconify-icon';
	import Directory from './Directory.svelte';
	import File from './File.svelte';
	import Loading from './Loading.svelte';
	import TasksList from './TasksList.svelte';
	import { uploadManager } from '$lib/files/upload/upload_manager.svelte';
	import type { DirFields, DragTarget, Entry, FileFields } from '$lib/files/entry';
	import { getFilesOverview, listFiles, moveFiles, type ListFilesResp } from '$lib/api/api_files';
	import { getSpaces } from '$lib/api/api_spaces';
	import type { Space } from '$lib/space';
	import type { SelectedEntry } from '$lib/files/select';
	import DeleteDialog from './DeleteDialog.svelte';
	import EntriesBar from './EntriesBar.svelte';
	import AddContentDialog from '$lib/files/AddContentDialog.svelte';
	import ContextMenu, { type ContextMenuProps } from '$lib/files/ContextMenu.svelte';
	import SpacesSelector from './SpacesSelector.svelte';
	import { client } from '$lib/stores/client.svelte';
	import RenameDialog from './RenameDialog.svelte';
	import MoveDialog from './MoveDialog.svelte';
	import EntriesOverviewNav from '$lib/files/overview/EntriesOverviewNav.svelte';

	const space = $derived(page.params.space!);
	const path = $derived(page.params.path!);

	let spaces: Space[] = $state([]);
	let current_space = $derived(spaces.find((s) => s.name === space));

	const fetchSpaces = async () => {
		const resp = await getSpaces(client);
		if (resp.status === 'success') spaces = resp.data!;
	};

	let filesResponse: ListFilesResp = $state({
		path: null,
		parent_dir: null,
		current_dir: '',
		entries: []
	});

	// File Overview

	// todo
	let overview_entries: any = $state();

	// todo: improve
	const fetchFilesOverview = async () => {
		const resp = await getFilesOverview(client, space, '', true);
		overview_entries = resp.data!.entries;
	};

	// File List Operations

	let entries: Entry[] = $derived(filesResponse.entries);
	let dir_entries = $derived(entries.filter((e) => e.entry_type === 'directory'));
	let file_entries = $derived(entries.filter((e) => e.entry_type === 'file'));
	// let file_entries = $derived.by(() => entries.filter((e) => e.entry_type === 'file'));

	let loading = $state(false);

	const getFilesList = async (path: string) => {
		loading = true;
		const resp = await listFiles(client, space, path);
		if (resp.status != 'success') {
			filesResponse = {
				path: null,
				parent_dir: null,
				current_dir: '',
				entries: []
			};
			loading = false;
			return;
		}
		filesResponse = resp.data!;
		loading = false;
	};

	const onListChange = async () => {
		// todo: should we clear the delete and selected list?
		await getFilesList(path);
	};

	// Select Operations

	let selected_entries: Set<Entry> = $derived(
		new Set(entries.filter((e) => e.is_selected === true))
	);
	let last_selected: SelectedEntry | undefined = $state();

	const handleSelectOp = async (e: MouseEvent, selected: SelectedEntry) => {
		// for now we will only allow selection across the same entry type
		// if shift then try selecting across
		if (
			e.shiftKey &&
			last_selected &&
			last_selected.entry.entry_type == selected.entry.entry_type
		) {
			let low_index = last_selected.index;
			let high_index = selected.index;
			if (last_selected.index > selected.index) {
				low_index = selected.index;
				high_index = last_selected.index;
			}

			entries.slice(low_index, high_index + 1).forEach((e) => {
				e.is_selected = true;
			});

			last_selected = selected;
			return;
		}

		// if not shift
		selected.entry.is_selected = !selected.entry.is_selected;
		last_selected = selected;
	};

	const handleDeselect = () => {
		entries
			.filter((e) => e.is_selected === true)
			.forEach((e) => {
				e.is_selected = false;
			});
	};

	// Self Drag Operations
	let drag_over_ct: number = $state(0);

	const onDragEnter = (e: DragEvent) => {
		if (!e.dataTransfer || e.dataTransfer.items.length < 1) {
			return;
		}
		// e.preventDefault();
		// e.stopImmediatePropagation();

		drag_over_ct++;
		if (drag_over_ct > 1) {
			return;
		}

		const items = [...e.dataTransfer.items];
		items.forEach((item) => {
			console.log(item.kind);
			console.log(item.type);
		});
	};

	const onDragOver = (e: DragEvent) => {
		if (drag_over_ct < 1) {
			return;
		}

		e.preventDefault();
	};

	const onDragLeave = (e: DragEvent) => {
		if (drag_over_ct < 1) {
			return;
		}

		drag_over_ct--;
		if (drag_over_ct === 0) {
			// todo
		}
	};

	const onDrop = (e: DragEvent) => {
		if (drag_over_ct < 1) {
			return;
		}
		e.preventDefault();
		const items = [...e.dataTransfer!.items];
		items.forEach((item) => {
			console.log(items);
		});

		drag_over_ct = 0;
	};

	const onDragEnd = (e: DragEvent) => {
		if (drag_over_ct < 1) {
			return;
		}

		drag_over_ct = 0;
	};

	// Entry Drag Operations
	let dragged_entries: Set<Entry> = $state(new Set());
	let entry_drag_target: DragTarget = $state({ entry: undefined, count: 0 });

	const onEntryDragStart = (e: DragEvent, entry: Entry) => {
		// single file being moved
		if (!selected_entries.has(entry)) {
			dragged_entries = new Set([entry]);
			return;
		}

		// multiple files being moved
		dragged_entries = selected_entries;
		console.log('todo: do UI stuff for multiple');
	};

	const onEntryDragEnd = (e: DragEvent, entry: Entry) => {
		dragged_entries = new Set();
	};

	const onEntryDragOver = (e: DragEvent, _: Entry) => {
		e.preventDefault();
	};

	const onEntryDragEnter = (e: DragEvent, entry: Entry) => {
		e.preventDefault();
		// todo: skip if selected, unless dir?
		if (dragged_entries.has(entry)) {
			return;
		}

		if (entry !== entry_drag_target.entry) {
			entry_drag_target.entry = entry;
			entry_drag_target.count = 0;
		}
		entry_drag_target.count++;
		entry.is_targetted = true;
	};

	const onEntryDragLeave = (e: DragEvent, entry: Entry) => {
		if (dragged_entries.has(entry)) {
			return;
		}

		entry_drag_target.count--;
		if (entry_drag_target.count === 0) {
			entry_drag_target.entry = undefined;
			entry.is_targetted = false;
		}
	};

	const onEntryDrop = async (e: DragEvent, entry: Entry) => {
		if (entry_drag_target.entry === undefined || dragged_entries.size < 1) {
			return;
		}

		let renames: [string, string][] = [];
		dragged_entries.forEach((e) => {
			renames.push([e.path, fs.join(entry.path, e.name)]);
			e.is_processing = true;
		});
		await handleMoveFiles(renames);
	};

	// Context Menu (right-click)

	// svelte-ignore non_reactive_update
	let contextMenuDialog: HTMLDialogElement;
	let contextMenuProps: { position: { x: number; y: number }; entry?: Entry } = $state({
		position: {
			x: 0,
			y: 0
		},
		entry: undefined
	});

	const handleContextMenu = (e: MouseEvent, entry?: Entry) => {
		e.preventDefault();
		e.stopPropagation();

		// Determine position
		// todo: improve
		const offset = 264;
		let x = e.pageX;
		let y = e.pageY;
		if (e.view && e.view!.innerWidth - x < offset) {
			x = e.view!.innerWidth - offset;
		}

		// Open or Move the menu
		contextMenuProps = {
			position: {
				x: x,
				y: y
			},
			entry: entry
		};
		if (entry) {
			entry.is_targetted = true;
		}
		contextMenuDialog.showPopover();
	};

	// CRUD Ops
	// svelte-ignore non_reactive_update
	let add_content_dialog: HTMLDialogElement;

	const handleAddContent = () => add_content_dialog.showModal();

	const handleMoveFiles = async (entries: [string, string][]) => {
		let resp = await moveFiles(client, space, entries);
		if (resp.status != 'success') {
			console.error(`could not move files: ${resp.message}`);
			return;
		}
		// todo: handle errors
		// const payload = await response.json();
		// todo: move this to onDrop?
		await onListChange();
	};

	// svelte-ignore non_reactive_update
	let move_entries_dialog: HTMLDialogElement;
	let target_move_entries: Set<Entry> = $state(new Set());

	const handleMoveEntries = (entry: Entry) => {
		// todo: include the right-clicked entry?
		target_move_entries = selected_entries;
		target_move_entries.add(entry);
		move_entries_dialog!.showModal();
	};

	// svelte-ignore non_reactive_update
	let delete_entries_dialog: HTMLDialogElement;
	let delete_entries: Entry[] = $state([]);

	const handleDeleteEntries = (entries: Entry[]) => {
		delete_entries = entries;
		delete_entries_dialog!.showModal();
	};

	const handleDeleteSelected = () => {
		if (selected_entries.size < 1) {
			return;
		}
		handleDeleteEntries(Array.from(selected_entries));
	};

	// svelte-ignore non_reactive_update
	let rename_entry_dialog: HTMLDialogElement;
	let target_rename_entry: Entry | undefined = $state();

	const handleRenameEntry = (entry: Entry) => {
		target_rename_entry = entry;
		rename_entry_dialog.showModal();
	};

	const onKeyDown = (e: KeyboardEvent) => {
		// `keydown` event is fired while the physical key is held down.

		// Assuming you only want to handle the first press, we early
		// return to skip.
		if (e.repeat) return;

		// In the switch-case we're updating our boolean flags whenever the
		// desired bound keys are pressed.

		switch (e.key) {
			case 'n':
				if (!e.altKey) {
					return;
				}
				handleAddContent();
			case 'Delete':
				handleDeleteSelected();
		}
	};

	$effect(() => {
		// temp
		uploadManager.upload_groups_completed;
		getFilesList(path);
		fetchFilesOverview();
		fetchSpaces();
	});
</script>

<svelte:window on:keydown={onKeyDown} />

<div class="files-view fx">
	<section class="left fx fx--col">
		<SpacesSelector {current_space} {spaces} />
		<EntriesOverviewNav {overview_entries} {space} />
	</section>
	<section class="right fx-grow fx fx--col">
		<EntriesBar
			{selected_entries}
			{add_content_dialog}
			{space}
			{handleDeleteSelected}
			{handleDeselect}
		/>
		<main
			class="entries fx-grow"
			class:drag-over={drag_over_ct > 0}
			class:loading
			ondragenter={onDragEnter}
			ondragover={onDragOver}
			ondragleave={onDragLeave}
			ondragend={onDragEnd}
			ondrop={onDrop}
			oncontextmenu={(e) => handleContextMenu(e)}
		>
			<section class="directories">
				<h3>Directories</h3>
				<div class="dir-list">
					{#each dir_entries as entry, index}
						<Directory
							{entry}
							{space}
							onSelect={(e: MouseEvent) => handleSelectOp(e, { index, entry })}
							onDragStart={onEntryDragStart}
							onDragEnd={onEntryDragEnd}
							onDragEnter={onEntryDragEnter}
							onDragOver={onEntryDragOver}
							onDragLeave={onEntryDragLeave}
							onDrop={onEntryDrop}
						/>
					{/each}
				</div>
			</section>
			<section class="files">
				<h3>Files</h3>
				<div class="file-list">
					{#each file_entries as entry, index}
						<File
							{entry}
							{space}
							onSelect={(e: MouseEvent) =>
								handleSelectOp(e, { index: index + dir_entries.length, entry: entry })}
							onDragStart={onEntryDragStart}
							onDragEnd={onEntryDragEnd}
							onDragOver={onEntryDragOver}
							onDrop={onEntryDrop}
							onContextMenu={handleContextMenu}
						/>
					{/each}
				</div>
			</section>
			<aside class="upload-bar fx fx--cc">
				<TasksList />
			</aside>
		</main>
	</section>
</div>

<AddContentDialog bind:dialog={add_content_dialog} {space} {onListChange} />
<MoveDialog
	bind:dialog={move_entries_dialog}
	{space}
	{path}
	{onListChange}
	entries={target_move_entries}
/>
<DeleteDialog bind:dialog={delete_entries_dialog} {space} {onListChange} entries={delete_entries} />
<RenameDialog bind:dialog={rename_entry_dialog} {space} {target_rename_entry} {onListChange} />
<ContextMenu
	bind:dialog={contextMenuDialog}
	position={contextMenuProps.position}
	{space}
	bind:entry={contextMenuProps.entry}
	{handleMoveEntries}
	{handleAddContent}
	{handleDeleteEntries}
	{handleRenameEntry}
/>

<style lang="scss">
	.files-view {
		border-top: 1px solid var(--clr-border);
	}

	.left {
		background-color: var(--clr-background-1);
		width: var(--sidebar-width);
		overflow-x: auto;
	}

	.right {
		border-left: 1px solid var(--clr-border);
		min-height: calc(100vh - var(--top-nav-height) - 1px);
	}

	// todo: redo this after moving the entities

	main.entries {
		background: var(--clr-background);
		padding: 1rem;
		position: relative;
		transition: opacity 0.2s;

		&.loading {
			opacity: 0.5;
		}

		&.drag-over {
			opacity: 0.5;
			// pointer-events: none;

			&::after {
				position: absolute;
				top: 0;
				left: 0;
				content: '';
				width: 100%;
				height: 100%;
				background: var(--clr-background);
				opacity: 0.4;
			}
		}

		> aside.upload-bar {
			position: fixed;
			padding-left: calc(var(--sidebar-width) + 1px);
			bottom: 0;
			left: 0;
			width: 100%;
		}

		> aside.add-action {
			position: fixed;
			right: 0;
			bottom: 0;
			padding: 1rem;
		}
	}

	.directories,
	.files {
		> h3 {
			// font-weight: normal;
			margin: 1rem 0;
		}

		&:first-of-type {
			> h3 {
				margin-top: 0;
			}
		}
	}

	.dir-list,
	.file-list {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(12rem, 1fr));
		grid-auto-rows: minmax(12rem, 1fr);
		// grid-auto-columns: max-content;
		gap: 0.75rem;
		// grid-template-columns: repeat(auto-fill);
	}

	.file-list {
		padding-bottom: 8rem;
	}

	// todo: move to module
	.directory {
		background-color: white;
		display: flex;
		border-radius: 3px;

		> .preview {
			margin: 0.5rem;
			font-size: 3rem;
			background-color: rgb(233, 235, 241);
			border-radius: 3px;

			> img {
				height: 3.5rem;
				width: 5rem;
			}
		}

		> .details {
			margin: 0 0.5rem 0.5rem 0.5rem;

			> h1 {
				font-size: 1rem;
				// font-weight: ;
				padding: 0;
				margin: 0;
			}
		}
	}
</style>
