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
	import TasksList from './TasksList.svelte';
	import { uploadManager } from '$lib/files/upload/upload_manager.svelte';
	import { EntryType } from '$lib/files/entry';
	import type { DirFields, DragTarget, Entry, FileFields } from '$lib/files/entry';
	import { downloadEntries } from '$lib/files/download';
	import { listFiles, moveFiles, type ListFilesResp } from '$lib/api/api_files';
	import type { SelectedEntry } from '$lib/files/select';
	import DeleteDialog from './DeleteDialog.svelte';
	import EntriesBar from '$lib/files/browse/EntriesBar.svelte';
	import AddContentDialog from '$lib/files/AddContentDialog.svelte';
	import ContextMenu, { type ContextMenuProps } from '$lib/files/ContextMenu.svelte';
	import MediaPreviewDialog from '$lib/files/MediaPreviewDialog.svelte';
	import { client } from '$lib/stores/client.svelte';
	import { useAppLoading } from '$lib/stores/loading.svelte';
	import { spaces, tryLoadSpaces } from '$lib/stores/spaces.svelte';
	import { tryRefreshOverview } from '$lib/stores/overview.svelte';
	import { tryRefreshShares } from '$lib/stores/shares.svelte';
	import RenameDialog from './RenameDialog.svelte';
	import MoveDialog from './MoveDialog.svelte';
	import { fsEntryIntoFiles } from '$lib/files/upload/drop';
	import { onMount } from 'svelte';
	import { UploadGroup } from '$lib/files/upload/upload_group';
	import { patchEntries } from '$lib/files/entries_patch';
	import { debounce, onSubscription } from '$lib/sse/sse.svelte';
	import { SubscriptionKind, type SseEntryEvent } from '$lib/sse/sse_event';

	const space = $derived(page.params.space!);
	const path = $derived(page.params.path!);

	let current_space = $derived(spaces.list.find((s) => s.name === space));
	let readonly = $derived(current_space?.readonly ?? true);

	const emptyListing = (): ListFilesResp => ({
		path: null,
		parent_dir: null,
		current_dir: '',
		entries: [],
		revision: 0
	});

	let filesResponse: ListFilesResp = $state(emptyListing());
	let revision: number | null = null;
	let pending_events: SseEntryEvent[] | null = null;
	let latest_fetch = 0;

	// File List Operations

	let entries: Entry[] = $derived(filesResponse.entries);
	let dir_entries = $derived(entries.filter((e) => e.entry_type === EntryType.DIRECTORY));
	let file_entries = $derived(entries.filter((e) => e.entry_type === EntryType.FILE));

	let loading = $state(false);
	let reloading = $state(false);

	useAppLoading(() => loading);

	const applyDirEvent = (ev: SseEntryEvent) => {
		if (pending_events) {
			pending_events.push(ev);
			return;
		}
		if (revision === null || ev.id <= revision) return;
		filesResponse.entries = patchEntries(filesResponse.entries, ev, path);
	};

	const getFilesList = async (path: string) => {
		const fetch_id = ++latest_fetch;
		revision = null;
		pending_events = [];

		const resp = await listFiles(client, space, path);
		if (fetch_id !== latest_fetch) return;

		const buffered = pending_events ?? [];
		pending_events = null;
		if (resp.status != 'success' || !resp.data) {
			filesResponse = emptyListing();
			return;
		}

		filesResponse = resp.data;
		revision = resp.data.revision;
		for (const ev of buffered) applyDirEvent(ev);
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
	const refreshFiles = async () => {
		loading = true;
		await getFilesList(path);
		loading = false;
	};

	const MEDIA_REFRESH_COOLDOWN_MS = 60_000;
	let last_media_refresh = 0;

	const onMediaError = debounce(() => {
		if (Date.now() - last_media_refresh < MEDIA_REFRESH_COOLDOWN_MS) return;
		last_media_refresh = Date.now();
		refreshFiles();
	});

	// Select Operations

	let in_selection = $state(false);

	let selected_entries: Set<Entry> = $derived(
		new Set(entries.filter((e) => e.is_selected === true))
	);
	let last_selected: SelectedEntry | undefined = $state();

	$effect(() => {
		if (in_selection && selected_entries.size === 0) in_selection = false;
	});

	const handleSelectOp = async (e: MouseEvent, selected: SelectedEntry) => {
		in_selection = true;

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
		in_selection = false;
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

		if (readonly) return;

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
			in_selection = true;
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

		// todo: move these to a file
		// todo: support mac
		switch (e.key) {
			case 'n':
				if (!e.altKey) {
					return;
				}
				if (readonly) return;
				handleAddContent();
				return;
			case 'r': {
				if (!e.altKey) {
					return;
				}
				if (readonly) return;
				if (selected_entries.size !== 1) return;
				const [entry] = selected_entries;
				handleRenameEntry(entry);
				return;
			}
			case 'Enter': {
				if (selected_entries.size !== 1) return;
				const [entry] = selected_entries;
				if (entry.entry_type !== EntryType.DIRECTORY) return;
				goto(`/${fs.join('files', space, entry.path)}`);
				return;
			}
			case 'Delete':
				if (readonly) return;
				handleDeleteSelected();
				return;
			case 'Escape':
				if (selected_entries.size > 0) exitSelection();
				return;
		}
	};

	onMount(() => {
		tryLoadSpaces(client);
	});

	$effect(() =>
		onSubscription(client, { kind: SubscriptionKind.DIR, space, path }, (ev) => {
			if (ev) return applyDirEvent(ev);
			if (filesResponse.path === path) return refreshFiles();
			reloadFiles();
		})
	);

	$effect(() => {
		const refresh = () => tryRefreshOverview(client, space);
		const debounced = debounce(refresh);
		return onSubscription(client, { kind: SubscriptionKind.TREE, space }, (ev) =>
			ev ? debounced() : refresh()
		);
	});

	$effect(() => {
		const refresh = () => tryRefreshShares(client, space);
		const debounced = debounce(refresh);
		return onSubscription(client, { kind: SubscriptionKind.SHARES, space }, (ev) =>
			ev ? debounced() : refresh()
		);
	});
</script>

<svelte:window on:keydown={onKeyDown} />

<div class="files-view fx">
	<section class="right fx-grow fx fx--col">
		<EntriesBar
			{selected_entries}
			{in_selection}
			{add_content_dialog}
			{space}
			{readonly}
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
			{onMediaError}
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

<AddContentDialog bind:dialog={add_content_dialog} {space} />
<MoveDialog bind:dialog={move_entries_dialog} {space} {path} entries={target_move_entries} />
<DeleteDialog bind:dialog={delete_entries_dialog} {space} entries={delete_entries} />
<RenameDialog bind:dialog={rename_entry_dialog} {space} {target_rename_entry} />
<ContextMenu
	bind:dialog={contextMenuDialog}
	position={contextMenuProps.position}
	bind:entry={contextMenuProps.entry}
	{space}
	onDownload={(entry) => downloadEntries(client, space, [entry])}
	// todo: Once RBAC lands, send a control object down instead
	handleMoveEntries={readonly ? undefined : handleMoveEntries}
	handleAddContent={readonly ? undefined : handleAddContent}
	handleDeleteEntries={readonly ? undefined : handleDeleteEntries}
	handleRenameEntry={readonly ? undefined : handleRenameEntry}
/>
<MediaPreviewDialog bind:this={preview} entries={preview_entries} {onMediaError} />

<style lang="scss">
	@use '$lib/styles/breakpoints' as bp;

	.right {
		min-height: calc(100vh - var(--top-nav-height) - 1px);
	}

	aside.upload-bar {
		position: fixed;
		padding-left: calc(var(--sidebar-width) + 1px);
		bottom: 0;
		left: 0;
		width: 100%;
		pointer-events: none;

		@media (max-width: bp.$bp-files-sidebar) {
			padding: 0 1rem;
			opacity: 0.9;
		}
	}
</style>
