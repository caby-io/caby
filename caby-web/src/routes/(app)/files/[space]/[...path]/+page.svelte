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
	import { goto } from '$app/navigation';
	import * as fs from '$lib/fs';

	import EntriesGrid from '$lib/files/browse/EntriesGrid.svelte';
	import LoadingBar from '$lib/LoadingBar.svelte';
	import TasksList from './TasksList.svelte';
	import { uploadManager } from '$lib/files/upload/upload_manager.svelte';
	import { EntryType } from '$lib/files/entry';
	import type { DirFields, DragTarget, Entry, FileFields } from '$lib/files/entry';
	import { downloadEntries } from '$lib/files/download';
	import { getFilesOverview, listFiles, moveFiles, type ListFilesResp } from '$lib/api/api_files';
	import { getSpaces } from '$lib/api/api_spaces';
	import type { Space } from '$lib/space';
	import type { SelectedEntry } from '$lib/files/select';
	import DeleteDialog from './DeleteDialog.svelte';
	import EntriesBar from '$lib/files/browse/EntriesBar.svelte';
	import AddContentDialog from '$lib/files/AddContentDialog.svelte';
	import ContextMenu, { type ContextMenuProps } from '$lib/files/ContextMenu.svelte';
	import MediaPreviewDialog from '$lib/files/MediaPreviewDialog.svelte';
	import SpacesSelector from './SpacesSelector.svelte';
	import { client } from '$lib/stores/client.svelte';
	import RenameDialog from './RenameDialog.svelte';
	import MoveDialog from './MoveDialog.svelte';
	import EntriesOverviewNav from '$lib/files/overview/EntriesOverviewNav.svelte';
	import { fsEntryIntoFiles } from '$lib/files/upload/drop';
	import { getContext } from 'svelte';

	const menu = getContext<{ open: boolean }>('menu');
	import { UploadGroup } from '$lib/files/upload/upload_group';

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
	let dir_entries = $derived(entries.filter((e) => e.entry_type === EntryType.DIRECTORY));
	let file_entries = $derived(entries.filter((e) => e.entry_type === EntryType.FILE));

	let loading = $state(false);
	let reloading = $state(false);

	const getFilesList = async (path: string) => {
		const resp = await listFiles(client, space, path);
		if (resp.status != 'success') {
			filesResponse = {
				path: null,
				parent_dir: null,
				current_dir: '',
				entries: []
			};
			return;
		}
		filesResponse = resp.data!;
	};

	// for first time loads and hard reloads of entire list
	const reloadFiles = async () => {
		loading = true;
		reloading = true;
		await getFilesList(path);
		reloading = false;
		loading = false;
	};

	// for atomic loads
	// note: this should be totally replaced by websockets eventually
	const refreshFiles = async () => {
		loading = true;
		await getFilesList(path);
		loading = false;
	};

	const onListChange = async () => {
		// todo: should we clear the delete and selected list?
		await refreshFiles();
		await fetchFilesOverview();
	};

	// Select Operations

	let selection_mode = $state<null | 'touch' | 'desktop'>(null);
	let in_selection = $derived(selection_mode !== null);

	let selected_entries: Set<Entry> = $derived(
		new Set(entries.filter((e) => e.is_selected === true))
	);
	let last_selected: SelectedEntry | undefined = $state();

	$effect(() => {
		if (selection_mode === 'desktop' && selected_entries.size === 0) selection_mode = null;
	});

	const handleSelectOp = async (e: MouseEvent, selected: SelectedEntry) => {
		if (selection_mode === null) selection_mode = 'desktop';

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

	// Media Preview

	let preview_entries = $derived(file_entries.filter((e) => e.entry_fields.can_preview));
	let preview: MediaPreviewDialog;

	const handlePreview = (entry: Entry) => {
		const index = preview_entries.indexOf(entry);
		if (index < 0) return;
		preview.openDialog(index);
	};

	const handleDeselect = () => {
		entries
			.filter((e) => e.is_selected === true)
			.forEach((e) => {
				e.is_selected = false;
			});
	};

	const exitSelection = () => {
		handleDeselect();
		selection_mode = null;
	};

	// Self Drag Operations

	let drag_over_ct: number = $state(0);

	const onDragEnter = (e: DragEvent) => {
		// filter to files
		if (!e.dataTransfer || e.dataTransfer.items.length < 1) {
			return;
		}
		// e.preventDefault();
		// e.stopImmediatePropagation();

		drag_over_ct++;
		if (drag_over_ct > 1) {
			return;
		}

		// todo: display something using these
		const items = [...e.dataTransfer.items];
		items.forEach((item) => {
			console.debug(item.kind);
			console.debug(item.type);
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

	const onDrop = async (e: DragEvent) => {
		if (drag_over_ct < 1) {
			return;
		}
		e.preventDefault();
		drag_over_ct = 0;

		// todo: webkitGetAsEntry -> getAsEntry in the future, code defensively
		const entries = [...e.dataTransfer!.items].flatMap((i) => i.webkitGetAsEntry() || []);

		await Promise.all(
			entries.map(async (entry) => {
				const files = await fsEntryIntoFiles(entry);
				if (files.length > 0) {
					uploadManager.addUploads(new UploadGroup(space, path, ...files));
				}
			})
		);
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
		// For mobile, go into selection mode instead. Browsers fire `contextmenu` as a
		// PointerEvent (subclass of MouseEvent) so we can read pointerType directly.
		const pointer_type = e instanceof PointerEvent ? e.pointerType : 'mouse';
		if (entry && (pointer_type === 'touch' || pointer_type === 'pen')) {
			e.preventDefault();
			e.stopPropagation();
			selection_mode = 'touch';
			entry.is_selected = true;
			const index = entries.findIndex((x) => x === entry);
			if (index >= 0) last_selected = { index, entry };
			if (typeof navigator !== 'undefined' && navigator.vibrate) navigator.vibrate(10);
			return;
		}

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

	const handleMoveSelected = () => {
		if (selected_entries.size < 1) return;
		target_move_entries = selected_entries;
		move_entries_dialog!.showModal();
	};

	const handleDownloadSelected = async () => {
		if (selected_entries.size < 1) return;
		await downloadEntries(client, space, Array.from(selected_entries));
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
				return;
			case 'Enter': {
				if (selected_entries.size !== 1) return;
				const [entry] = selected_entries;
				if (entry.entry_type !== EntryType.DIRECTORY) return;
				goto(`/${fs.join('files', space, entry.path)}`);
				return;
			}
			case 'Delete':
				handleDeleteSelected();
				return;
			case 'Escape':
				if (selected_entries.size > 0) exitSelection();
				return;
		}
	};

	$effect(() => {
		reloadFiles();
		fetchFilesOverview();
		fetchSpaces();
	});

	let last_upload_completed = uploadManager.upload_groups_completed;
	$effect(() => {
		const completed = uploadManager.upload_groups_completed;
		if (completed === last_upload_completed) return;
		last_upload_completed = completed;
		refreshFiles();
		fetchFilesOverview();
	});
</script>

<svelte:window on:keydown={onKeyDown} />

<LoadingBar {loading} />
<div class="files-view fx">
	<section class="left fx fx--col" class:open={menu.open}>
		<SpacesSelector {current_space} {spaces} />
		<EntriesOverviewNav {overview_entries} {space} />
	</section>
	<div
		class="menu-backdrop"
		class:open={menu.open}
		role="presentation"
		onclick={() => (menu.open = false)}
	></div>
	<section class="right fx-grow fx fx--col">
		<EntriesBar
			{selected_entries}
			{in_selection}
			{add_content_dialog}
			{space}
			{handleDeleteSelected}
			{handleMoveSelected}
			{handleDownloadSelected}
			{exitSelection}
		/>
		<EntriesGrid
			{dir_entries}
			{file_entries}
			{space}
			{in_selection}
			dimmed={reloading}
			{drag_over_ct}
			{onDragEnter}
			{onDragOver}
			{onDragLeave}
			{onDragEnd}
			{onDrop}
			{handleContextMenu}
			{handleSelectOp}
			{handlePreview}
			{onEntryDragStart}
			{onEntryDragEnd}
			{onEntryDragEnter}
			{onEntryDragOver}
			{onEntryDragLeave}
			{onEntryDrop}
		/>
		<aside class="upload-bar fx fx--cc">
			<TasksList />
		</aside>
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
	bind:entry={contextMenuProps.entry}
	{space}
	onDownload={(entry) => downloadEntries(client, space, [entry])}
	{handleMoveEntries}
	{handleAddContent}
	{handleDeleteEntries}
	{handleRenameEntry}
/>
<MediaPreviewDialog bind:this={preview} entries={preview_entries} />

<style lang="scss">
	@use '$lib/styles/breakpoints' as bp;

	.left {
		background-color: var(--clr-background-1);
		width: var(--sidebar-width);
		overflow-x: auto;

		@media (max-width: bp.$bp-files-sidebar) {
			position: fixed;
			top: var(--top-nav-height);
			left: 0;
			bottom: 0;
			z-index: 2;
			transform: translateX(-100%);
			visibility: hidden;
			transition:
				transform 0.25s ease,
				visibility 0s linear 0.25s;

			&.open {
				transform: translateX(0);
				visibility: visible;
				transition:
					transform 0.25s ease,
					visibility 0s linear 0s;
			}
		}
	}

	.menu-backdrop {
		display: none;

		@media (max-width: bp.$bp-files-sidebar) {
			display: block;
			position: fixed;
			inset: var(--top-nav-height) 0 0 0;
			background: rgba(0, 0, 0, 0.4);
			z-index: 1;
			opacity: 0;
			pointer-events: none;
			transition: opacity 0.25s ease;

			&.open {
				opacity: 1;
				pointer-events: auto;
			}
		}
	}

	.right {
		border-left: 1px solid var(--clr-border);
		min-height: calc(100vh - var(--top-nav-height) - 1px);
	}

	aside.upload-bar {
		position: fixed;
		padding-left: calc(var(--sidebar-width) + 1px);
		bottom: 0;
		left: 0;
		width: 100%;

		@media (max-width: bp.$bp-files-sidebar) {
			padding: 0 1rem;
			opacity: 0.9;
		}
	}
</style>
