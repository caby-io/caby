<script lang="ts">
	import { page } from '$app/state';
	import Dialog from '$lib/Dialog.svelte';
	import NewFolderDialog from '../../routes/(app)/files/[...path]/NewFolderDialog.svelte';
	import { UploadGroup } from './upload/upload_group';
	import { uploadManager } from './upload/upload_manager.svelte';

	let { dialog = $bindable(), onListChange }: { dialog: HTMLDialogElement; onListChange: any } =
		$props();
	const path = $derived(page.params.path!);

	const openAddDialog = () => {
		dialog.showModal();
	};

	const handleUploadFiles = async (files: FileList) => {
		// for now we are always making an upload group for each file
		for (const file of files) {
			uploadManager.addUploads(new UploadGroup(path, file));
		}
	};

	const openFileDialog = (dir: boolean = false) => {
		const input = document.createElement('input');
		input.setAttribute('type', 'file');
		input.setAttribute('multiple', 'multiple');
		if (dir) {
			input.setAttribute('webkitdirectory', 'webkitdirectory');
		}

		input.onchange = (_) => {
			handleUploadFiles(input.files!);
			dialog.close();
		};
		input.click();
	};

	// New directory
	// svelte-ignore non_reactive_update
	let newFolderDialog: HTMLDialogElement;

	const openNewFolderDialog = () => {
		newFolderDialog.showModal();
		dialog.close();
	};
</script>

<Dialog bind:dialog title={undefined}>
	{#snippet content()}
		<div class="add-list">
			<button class="fx fx--ac border-0 box-shadow-0-card" onclick={() => openNewFolderDialog()}>
				<div class="fx fx--cc">
					<iconify-icon icon="flat-color-icons:folder"></iconify-icon>
					<span class="overlay"><iconify-icon icon="lucide:plus"></iconify-icon></span>
				</div>
				New Folder
			</button>
			<button class="fx fx--ac border-0 box-shadow-0-card" disabled>
				<div class="fx fx--cc">
					<iconify-icon icon="streamline-flex-color:text-file-flat"></iconify-icon>
					<span class="overlay"><iconify-icon icon="lucide:plus"></iconify-icon></span>
				</div>
				New File (coming soon)
			</button>
			<button class="fx fx--ac border-0 box-shadow-0-card" onclick={() => openFileDialog(true)}>
				<div class="fx fx--cc">
					<iconify-icon icon="flat-color-icons:folder"></iconify-icon>
					<span class="overlay"><iconify-icon icon="ph:upload-fill"></iconify-icon></span>
				</div>
				Upload Directories
			</button>
			<button class="fx fx--ac border-0 box-shadow-0-card" onclick={() => openFileDialog(false)}>
				<div class="fx fx--cc">
					<iconify-icon icon="streamline-flex-color:text-file-flat"></iconify-icon>
					<span class="overlay"><iconify-icon icon="ph:upload-fill"></iconify-icon></span>
				</div>
				Upload Files
			</button>
			<button class="fx fx--ac border-0 box-shadow-0-card" disabled>
				<div class="fx fx--cc youtube"><iconify-icon icon="logos:youtube-icon"></iconify-icon></div>
				Fetch Youtube Audio<br /> (coming soon)
			</button>
		</div>
	{/snippet}
</Dialog>

<NewFolderDialog bind:dialog={newFolderDialog} {onListChange} />

<style lang="scss">
	.add-button {
		cursor: pointer;
		background-color: var(--clr-primary);
		color: var(--clr-background);
		font-size: 1.6rem;
		width: 2.2rem;
		height: 2.2rem;
		border-radius: 50%;
	}

	.add-list {
		display: grid;
		grid-template-columns: 1fr 1fr;
		grid-auto-rows: 5rem;
		gap: 0.5rem;
		padding: 1rem;

		> button {
			cursor: pointer;
			background: var(--clr-background-2);
			padding: 1rem;

			&:disabled {
				cursor: not-allowed;
				opacity: 0.4;
			}

			> div {
				position: relative;
				font-size: 2rem;
				margin-right: 1rem;
				width: 2rem;

				iconify-icon {
					// filter: drop-shadow(2px 4px 6px var(--clr-shadow));
					filter: drop-shadow(2px 2px 3px var(--clr-shadow));
				}

				&.youtube {
					font-size: 1rem;
				}

				> span.overlay {
					color: var(--clr-primary);
					display: inherit;
					position: absolute;
					right: -0.6em;
					bottom: -0.4em;
					font-size: 0.6em;
				}
			}
		}
	}
</style>
