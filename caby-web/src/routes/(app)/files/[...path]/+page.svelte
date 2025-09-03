<script module lang="ts">
	export const enum MoveOp {
		ADD_SRC,
		REM_SRC,
		// SET_DST,
		// REM_DST,
		EXEC
	}

	export type UploadRegistration = {
		id: string;
		chunk_size: number;
		token: string;
	};
</script>

<script lang="ts">
	import { page } from '$app/stores';
	import { on } from 'svelte/events';
	import * as fs from '$lib/fs';

	import 'iconify-icon';
	import Directory, { type DirEntry } from './Directory.svelte';
	import File from './File.svelte';
	import Loading from './Loading.svelte';
	import UploadBar from './UploadBar.svelte';
	import AddAction from './AddAction.svelte';
	import { uploadManager } from '$lib/files/upload_manager.svelte';
	import { TaskStatus } from '$lib/files/upload_file.svelte';

	type FilesResponse = {
		path: string | null;
		parent_dir: string | null;
		current_dir: string;
		entries: Array<Entry>;
	};

	let onuploadComplete = $props();

	let filesResponse: FilesResponse = $state({
		path: null,
		parent_dir: null,
		current_dir: '',
		entries: []
	});

	type FilesRender = {
		entries: Array<Entry | undefined>;
	};

	let filesRender: FilesRender = $state({
		entries: []
	});

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

		// temp?
		// virtualizeList();

		// Fix URL if it's incorrect
		// if document.location.href != join("files", response.path) {
		// 	document.location.href = join("files", response.path)
		// }

		loading = false;
		// document.location.href = join("files", response.path)
	};

	const virtualizeList = () => {
		// const filesList = document.getElementById('files-list');
		const main = document.getElementById('main');
		const height = window.innerHeight - (main!.offsetTop || 0);
		const offset = main!.scrollTop;

		// todo: get dynamically
		const trHeadHeight = 44.17;
		const trBodyHeight = 72.33;

		// we won't bother calculating when the bottom-most element appears
		const maxTR = Math.ceil(height / trBodyHeight);
		// todo: should be +1 when we have parent dir link
		let elOffset = Math.floor(offset / trBodyHeight - trHeadHeight / trBodyHeight);

		const entryCount = filesResponse.entries.length;

		filesRender.entries = Array(entryCount).fill(undefined);

		if (elOffset < 1) {
			elOffset = 0;
		}

		const filesOffset = elOffset - entryCount;
		// console.log(filesOffset);

		// Dirs
		const dirsRendered = Math.min(elOffset + maxTR, entryCount - 1) - elOffset;
		for (let i = elOffset; i <= Math.min(elOffset + maxTR, entryCount - 1); i++) {
			filesRender.entries[i] = filesResponse.entries[i];
		}
	};

	const deleteFiles = async (paths: [string]) => {
		const response = await fetch('http://localhost:8080/v0/files/delete', {
			method: 'post',
			headers: {
				Accept: 'application/json',
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				entries: paths,
				force: false
			})
		});
		// todo: handle errors
		const payload = await response.json();
		await getFilesList($page.params.path);
	};

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
		await getFilesList($page.params.path);
	};

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

	let dragged_entries: Set<Entry> = $state(new Set());
	// let targetEntry: Entry | undefined = $state();

	const handleMoveOp = async (operation: MoveOp, entry: Entry) => {
		switch (operation) {
			case MoveOp.ADD_SRC:
				dragged_entries.add(entry);
				break;
			case MoveOp.REM_SRC:
				dragged_entries.delete(entry);
				break;
			// case MoveOp.SET_DST:
			// 	targetEntry = entry;
			// 	break;
			// case MoveOp.REM_DST:
			// 	targetEntry = undefined;
			// 	break;
			case MoveOp.EXEC:
				if (dragged_entries.size < 1) {
					console.error('missing destination');
					return;
				}

				let renames: [string, string][] = [];

				dragged_entries.forEach((e) => {
					renames.push([e.path, fs.join(entry.path, e.name)]);
				});

				await moveFiles(renames);
				break;
		}
	};

	$effect(() => {
		// TEMP
		uploadManager.upload_groups_completed;
		getFilesList($page.params.path);
		// getFilesList($page.params.path);
	});

	// $effect(() => {
	// 	virtualizeList();
	// });
</script>

<div class="files-view fx">
	<section class="left">left</section>
	<section class="right fx-grow fx fx--col">
		<header class="fx fx--ac">
			<div class="breadcrumbs fx fx--ac">
				<div class="breadcrumb fx fx--ac">
					<a class="fx fx--ac" href="#">
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
					{#each filesResponse.entries.filter((e) => e?.entry_type === 'directory') as entry}
						<Directory
							{entry}
							onDelete={(path: string) => deleteFiles([path])}
							onRename={(entry: DirEntry) => renameEntryDialog(entry)}
							{handleMoveOp}
						/>
					{/each}
				</div>
			</section>
			<section class="files">
				<h3>Files</h3>
				<div class="file-list">
					{#each filesResponse.entries.filter((e) => e?.entry_type === 'file') as entry}
						<File
							{entry}
							isSelected={entry.isSelected}
							onSelect={() => (entry.isSelected = !entry.isSelected)}
						/>
					{/each}
				</div>
			</section>
			<aside class="upload-bar fx fx--cc">
				<UploadBar />
			</aside>
			<aside class="add-action fx fx--cc">
				<AddAction />
			</aside>
		</main>
	</section>
</div>

<dialog id="rename-modal" class="rename-modal">
	<header class="fx">
		<h2>Rename</h2>
		<button
			class="close fx fx-cc"
			onclick={() => {
				let dialog: HTMLDialogElement | null = document.querySelector('#rename-modal');
				dialog!.close();
			}}
		>
			<iconify-icon icon="lucide:x"></iconify-icon>
		</button>
	</header>
	<main>
		<p>Renaming '<span>{renameEntry.srcName}</span>'</p>
		<input type="text" bind:value={renameEntry.dstName} />
	</main>
	<footer class="fx fx-cc">
		<button
			class="cancel"
			onclick={() => {
				let dialog: HTMLDialogElement | null = document.querySelector('#rename-modal');
				dialog!.close();
			}}>Cancel</button
		>
		<button
			onclick={() => {
				moveFiles([
					[renameEntry.srcPath, fs.join(fs.parent(renameEntry.srcPath), renameEntry.dstName)]
				]);
				let dialog: HTMLDialogElement | null = document.querySelector('#rename-modal');
				dialog!.close();
			}}
		>
			Rename
		</button>
	</footer>
</dialog>

<style lang="scss">
	.files-view {
		border-top: 1px solid var(--clr-border);
	}

	.left {
		background-color: var(--clr-background-1);
		width: var(--sidebar-width);
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
