<script lang="ts">
	import { page } from '$app/stores';
	import * as fs from '$lib/fs';

	import 'iconify-icon';
	import Directory, { type DirEntry } from './Directory.svelte';
	import File, { type FileEntry } from './File.svelte';
	import Loading from './Loading.svelte';

	type FilesResponse = {
		path: string | null;
		parent_dir: string | null;
		current_dir: string;
		entries: Array<Entry>;
	};

	type Entry = {
		entry_type: string;
		name: string;
		path: string;
		created_at: string;
		pretty_created_at: string;
		modified_at: string;
		pretty_modified_at: string;
		entry_fields: any; // todo
	};

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

	const get_data = async (path: string) => {
		loading = true;
		const response = await fetch('http://localhost:8080/v0/files/' + path);
		const payload = await response.json();

		filesResponse = payload.data;

		// temp?
		virtualizeList();

		// Fix URL if it's incorrect
		// if document.location.href != join("files", response.path) {
		// 	document.location.href = join("files", response.path)
		// }

		loading = false;
		// document.location.href = join("files", response.path)
	};

	const join = (...paths: Array<string>): string => {
		let joined = '';
		paths
			.filter((p) => p != '' && p != '/' && p != null)
			.forEach((p) => {
				while (p.charAt(0) === '/') {
					p = p.substring(1);
				}
				joined += `/${p}`;
			});
		return joined;
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

	const deleteEntries = async (paths: [string]) => {
		const response = await fetch('http://localhost:8080/v0/files', {
			method: 'delete',
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
		await get_data($page.params.path);
	};

	const renameEntries = async (entries: [[string, string]]) => {
		const response = await fetch('http://localhost:8080/v0/files/rename', {
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
		await get_data($page.params.path);
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

	$effect(() => {
		get_data($page.params.path);
	});

	// $effect(() => {
	// 	virtualizeList();
	// });
</script>

<div class="right fx fx--col fx-grow">
	<header>breadcrumbs</header>
	<section class="file-list">
		<section class="top-bar">
			<div class="location"></div>
		</section>
		<main class="entries" id="main" onscroll={virtualizeList}>
			{#if loading}
				<table id="files-list" class="skeleton">
					<thead>
						<tr>
							<th class="icon"></th>
							<th class="name"><span /></th>
							<th class="actions"><span /></th>
							<th><span /></th>
						</tr>
					</thead>
					<tbody>
						{#each { length: 3 } as _, i}
							<tr>
								<th class="icon"></th>
								<th class="name"><span /></th>
								<th class="actions"><span /></th>
								<th><span /></th>
							</tr>
						{/each}
					</tbody>
				</table>
			{:else}
				<table id="files-list">
					<thead>
						<tr>
							<th class="icon"></th>
							<th class="name">Name</th>
							<th class="actions"></th>
							<th>Last Modified</th>
						</tr>
					</thead>
					<tbody>
						<!-- Parent Dir -->
						{#if filesResponse.parent_dir != null}
							<tr>
								<td data-cell="select" class="check"></td>
								<td data-cell="main" class="main fx">
									<div class="icon fx fx-cc">
										<a href={`${join('files', filesResponse.parent_dir)}`}>📁</a>
									</div>
									<div class="text fx-grow">
										<div class="name">
											<a href={`${join('files', filesResponse.parent_dir)}`}>..</a>
										</div>
										<!-- <div class="size">Unknown</div> -->
									</div>
								</td>
								<td data-cell="actions">..</td>
								<td data-cell="last-modified"></td>
							</tr>
						{/if}
						<!-- Entries -->
						{#each filesRender.entries as entry}
							{#if entry?.entry_type == 'directory'}
								<Directory
									dir_entry={entry}
									onDelete={(path: string) => deleteEntries([path])}
									onRename={(entry: DirEntry) => renameEntryDialog(entry)}
								/>
							{:else if entry?.entry_type == 'file'}
								<File
									file_entry={entry}
									onDelete={(path: string) => deleteEntries([path])}
									onRename={(entry: FileEntry) => renameEntryDialog(entry)}
								/>
							{:else}
								<Loading />
								<!-- <tr style="height: 72.33px"></tr> -->
							{/if}
						{/each}
					</tbody>
				</table>
			{/if}
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
				renameEntries([
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
	.right {
		max-height: 100vh;
	}

	.file-list {
		flex-grow: 1;
		height: 0; /* Need to investigate why this works */
	}

	.entries {
		// margin: 1rem;
		flex-grow: 1;
		height: 100%;
		overflow-y: scroll;

		> table {
			border-collapse: collapse;
			// border-spacing: 0rem;
			font-size: 1.1em;
			width: 100%;

			&.skeleton {
				span {
					display: block;
					height: 0.4rem;
					min-width: 100px;
					width: 70%;
					background: lightgrey;
					border-radius: 3px;
				}
			}

			// General
			tr {
				border-radius: 3px;
			}
			td,
			th {
				padding: 0.5rem;
				text-align: left;
			}

			td.check {
				width: 2rem;
			}

			td.main {
				display: flex;

				.icon {
					font-size: 1.75em;
					width: 2em;
					position: relative;

					.indicator {
						position: absolute;
						display: inline-flex;
						bottom: 0.25rem;
						right: 0.25rem;
						background: rgba(255, 255, 255, 0.8);
						border-radius: 50%;
						padding: 0.2rem;

						&--symlink {
							iconify-icon {
								font-size: 1rem;
							}
						}

						&--broken-symlink {
							iconify-icon {
								font-size: 1rem;
								color: red;
							}
						}
					}

					a {
						text-decoration: none;
					}
				}

				.name {
					width: 60%;
				}
			}

			td.actions {
				font-size: 1.5rem;

				.action {
					cursor: pointer;
					color: var(--clr-secondary);
					margin-right: 0.5rem;
					width: 2.1rem;
					height: 2.1rem;
					background: var(--clr-accent);
					opacity: 0.6;
					border-radius: 3px;
					transition: color 0.3s;

					&--invisible {
						opacity: 0 !important;
						pointer-events: none;
					}
				}
				// > div {
				// 	display: inline-block;
				// 	padding: 2px;
				// 	margin-right: 0.5rem;
				// 	background: lightgrey;
				// }
			}

			tbody > tr {
				&:hover {
					color: var(--clr-background);
					background-color: var(--clr-secondary);

					a {
						color: var(--clr-background);
					}

					td.actions .action {
						opacity: 0.8;
					}
				}

				td.actions .action:hover {
					opacity: 1;
					color: var(--clr-primary);
				}
			}
		}

		// > div {
		// 	display: flex;
		// 	// border: 1px solid var(--clr-accent);
		// 	transition:
		// 		background-color 0.3s,
		// 		color 0.3s;
		// 	font-size: 1.2em; // TEMP
		// 	padding: 0.5rem;
		// 	border-radius: 3px;

		// 	> .icon {
		// 		width: 1.5em;
		// 	}

		// 	&:hover {
		// 		color: var(--clr-background);
		// 		background-color: var(--clr-secondary);

		// 		a {
		// 			color: var(--clr-background);
		// 		}
		// 	}
		// }
	}

	.rename-modal {
		&::backdrop {
			background: var(--clr-background);
			opacity: 0.5;
			backdrop-filter: blur(2px);
		}

		margin: auto;
		background: var(--clr-background);
		color: var(--clr-primary);
		padding: 0;
		min-width: 30rem;
		max-width: 80vw;
		border: 0;
		box-shadow: 0 0 1em rgb(0 0 0 / 0.3);

		> header {
			background: var(--clr-primary);
			color: var(--clr-background);
			padding: 0.5rem 1rem;

			> h2 {
				font-weight: normal;
			}

			> .close {
				cursor: pointer;
				background: none;
				padding: 0;
				border: none;
				font-size: 1.5em;
				margin-left: auto;
			}
		}

		> main {
			padding: 1.5em;

			> p > span {
				font-weight: bold;
				// color: var(--clr-primary);
			}
		}

		input {
			outline: none;
			border: none;
			border-bottom: 2px solid var(--clr-primary);
			border-radius: 3px 3px 1px 1px;
			background: var(--clr-secondary-background);
			color: var(--clr-text);
			font-size: 1.1em;
			padding: 0.5em;
		}

		> footer {
			background: var(--clr-secondary-background);
			padding: 0.5rem 1rem;
			justify-content: flex-end;
			gap: 0.5em;

			button {
				cursor: pointer;
				font-size: 1em;
				background: var(--clr-primary);
				border: none;
				border-radius: 2px;
				padding: 0.5rem 1rem;

				&.cancel {
					background: none;
					color: var(--clr-primary);
				}
			}
		}
	}
</style>
