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
	import UploadBar from './UploadBar.svelte';
	import AddAction from './AddAction.svelte';
	import { uploadManager } from '$lib/files/upload/upload_manager.svelte';
	import type { DirFields, DragTarget, Entry, FileFields } from '$lib/files/entry';
	import type { FilesResponse } from '$lib/api/api_files';
	import type { SelectedEntry } from '$lib/files/select';
	import DeleteDialog from './DeleteDialog.svelte';

	const path = $derived(page.params.path!);

	let filesResponse: FilesResponse = $state({
		path: null,
		parent_dir: null,
		current_dir: '',
		entries: []
	});

	let entries: Entry[] = $derived(filesResponse.entries);
	let dir_entries = $derived(entries.filter((e) => e.entry_type === 'directory'));
	let file_entries = $derived(entries.filter((e) => e.entry_type === 'file'));

	// File List Operations

	let loading = $state(false);

	const getFilesList = async (path: string) => {
		loading = true;
		const response = await fetch('http://localhost:8080/v0/files/list/' + path);
		const payload = await response.json();

		let data = payload.data;
		data.entries.forEach((d: any) => {
			d.selected = false;
		});

		filesResponse = data;

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

		// if shift then try selecting across
		// if not shift
		selected.entry.is_selected = !selected.entry.is_selected;
		last_selected = selected;
	};

	// Drag Operations

	let dragged_entries: Set<Entry> = $state(new Set());
	let drag_target: DragTarget = $state({ entry: undefined, count: 0 });
	let targetEntry: Entry | undefined = $state();

	const onDragStart = (e: DragEvent, entry: Entry) => {
		// single file being moved
		if (!selected_entries.has(entry)) {
			dragged_entries = new Set([entry]);
			return;
		}

		// multiple files being moved
		dragged_entries = selected_entries;
		console.log('todo: do UI stuff for multiple');
	};

	const onDragEnd = (e: DragEvent, entry: Entry) => {
		dragged_entries = new Set();
	};

	const onDragOver = (e: DragEvent, _: Entry) => {
		e.preventDefault();
	};

	const onDragEnter = (e: DragEvent, entry: Entry) => {
		e.preventDefault();
		// todo: skip if selected, unless dir?
		if (dragged_entries.has(entry)) {
			return;
		}

		if (entry !== drag_target.entry) {
			drag_target.entry = entry;
			drag_target.count = 0;
		}
		drag_target.count++;
	};

	const onDragLeave = (e: DragEvent, entry: Entry) => {
		if (dragged_entries.has(entry)) {
			return;
		}

		drag_target.count--;
		if (drag_target.count === 0) {
			drag_target.entry = undefined;
		}
	};

	const onDrop = async (e: DragEvent, entry: Entry) => {
		if (drag_target.entry === undefined || dragged_entries.size < 1) {
			return;
		}

		let renames: [string, string][] = [];
		dragged_entries.forEach((e) => {
			renames.push([e.path, fs.join(entry.path, e.name)]);
		});
		await moveFiles(renames);
		console.log('execute move');
	};

	// CRUD Ops

	const moveFiles = async (entries: [string, string][]) => {
		const response = await fetch('http://localhost:8080/v0/files/move', {
			method: 'post',
			headers: {
				Accept: 'application/json',
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				entries,
				force: false
			})
		});
		// todo: handle errors
		const payload = await response.json();
		// todo: move this to onDrop?
		await onListChange();
	};

	// svelte-ignore non_reactive_update
	let delete_entries_dialog: HTMLDialogElement;
	let delete_entries: Entry[] = $state([]);

	const handleDeleteSelected = () => {
		if (selected_entries.size < 1) {
			return;
		}
		delete_entries = Array.from(selected_entries);
		delete_entries_dialog!.showModal();
	};

	const onKeyDown = (e: KeyboardEvent) => {
		// `keydown` event is fired while the physical key is held down.

		// Assuming you only want to handle the first press, we early
		// return to skip.
		if (e.repeat) return;

		// In the switch-case we're updating our boolean flags whenever the
		// desired bound keys are pressed.

		switch (e.key) {
			case 'Delete':
				handleDeleteSelected();
		}
	};

	$effect(() => {
		// uploadManager.upload_groups_completed;
		getFilesList(path);
	});
</script>

<svelte:window on:keydown={onKeyDown} />

<div class="files-view fx">
	<section class="left fx fx--col">
		<button class="add-button button">Add</button>
	</section>
	<section class="right fx-grow fx fx--col">
		<header class="fx fx--ac">
			<div class="breadcrumbs fx fx--ac">
				<div class="breadcrumb fx fx--ac">
					<a class="fx fx--ac" href="/files">
						<iconify-icon icon="ci:house-02"></iconify-icon>
					</a>
				</div>
				<div class="breadcrumb fx fx--ac">
					<a class="fx fx--ac" href="#">Folder A</a>
				</div>
				<div class="breadcrumb fx fx--ac">
					<a class="fx fx--ac" href="#">Folder B</a>
				</div>
			</div>
		</header>
		<main class="entries fx-grow">
			<section class="directories">
				<h3>Directories</h3>
				<div class="dir-list">
					{#each dir_entries as entry, index}
						<Directory
							{entry}
							onSelect={(e: MouseEvent) => handleSelectOp(e, { index, entry })}
							{onDragStart}
							{onDragEnd}
							{onDragEnter}
							{onDragOver}
							{onDragLeave}
							{onDrop}
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
							onSelect={(e: MouseEvent) =>
								handleSelectOp(e, { index: index + dir_entries.length, entry: entry })}
							{onDragStart}
							{onDragEnd}
							{onDragOver}
							{onDrop}
						/>
					{/each}
				</div>
			</section>
			<aside class="upload-bar fx fx--cc">
				<UploadBar />
			</aside>
			<aside class="add-action fx fx--cc">
				<AddAction {onListChange} />
			</aside>
		</main>
	</section>
</div>

<DeleteDialog bind:dialog={delete_entries_dialog} {onListChange} entries={delete_entries} />

<style lang="scss">
	.files-view {
		border-top: 1px solid var(--clr-border);
	}

	.left {
		background-color: var(--clr-background-1);
		min-width: var(--sidebar-width);

		.add-button {
			margin: 1rem;
			text-align: center;
		}
	}

	.right {
		border-left: 1px solid var(--clr-border);
		min-height: calc(100vh - var(--top-nav-height) - 1px);

		> header {
			height: 2.6rem;
			background-color: var(--clr-background-1);
			border-bottom: 1px solid var(--clr-border);
			padding: 0 1rem;
		}

		.breadcrumbs {
			.breadcrumb > a {
				// display: block;
				padding: 0 0.25rem;
				height: 2rem;
				text-decoration: none;
				transition: opacity 0.2s;
				opacity: 1;
				border-radius: 3px;

				&:hover {
					color: inherit;
					opacity: 0.7;
				}
			}

			div + div:before {
				font-family: serif;
				font-weight: bold;
				display: block;
				content: '/';
				margin: 0.25rem;
				opacity: 0.4;
			}
		}
	}

	// todo: redo this after moving the entities

	main.entries {
		background: var(--clr-background);
		padding: 1rem;
		position: relative;

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
