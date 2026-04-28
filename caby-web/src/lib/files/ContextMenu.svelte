<script module>
	import { getDownloadURL } from '$lib/api/api_files';
	import { client } from '$lib/stores/client.svelte';
	import type { Entry } from './entry';

	export type ContextMenuProps = {
		dialog: HTMLElement;
		position: { x: number; y: number };
		space: string;
		entry?: Entry;

		handleAddContent: any;
		handleMoveEntries: any;
		handleDeleteEntries: any; // todo
		handleRenameEntry: any;
	};
</script>

<script lang="ts">
	let {
		dialog = $bindable(),
		position,
		space,
		entry = $bindable(),
		handleAddContent,
		handleMoveEntries,
		handleDeleteEntries,
		handleRenameEntry
	}: ContextMenuProps = $props();

	const isDir = $derived(entry?.entry_type === 'directory');
	const typeName = $derived(isDir ? 'Folder' : 'File');

	// todo: check that this isn't too expensive
	const handleWindowClick = (e: MouseEvent) => {
		if (dialog.contains(e.target as Node)) {
			return;
		}
		dialog.hidePopover();
	};

	const onbeforetoggle = (e: ToggleEvent) => {
		if (!entry) {
			return;
		}

		if (e.newState === 'open') {
			entry.is_targetted = true;
			return;
		}
		entry.is_targetted = false;
	};

	// const oncontextmenu = (e: MouseEvent) => {
	// 	e.preventDefault();
	// 	dialog.hidePopover();
	// };

	const handleDownload = async () => {
		if (!entry) return;
		const url = await getDownloadURL(client, space, [entry]);
		if (!url) {
			console.error('could not get download url');
			return;
		}
		const a = document.createElement('a');
		a.href = url;
		a.download = '';
		document.body.appendChild(a);
		a.click();
		a.remove();
	};
</script>

<svelte:window onclick={handleWindowClick} />

<div
	bind:this={dialog}
	class="context-menu border-0 box-shadow-0-card"
	style="left: {position.x}px; top: {position.y}px"
	popover
	{onbeforetoggle}
>
	<section class="context-menu-container fx fx--col">
		{#if !entry || isDir}
			<button class="context-item fx" onclick={() => handleAddContent([entry])}>
				<div class="icon fx fx--cc">
					<iconify-icon icon="lucide:plus"></iconify-icon>
				</div>
				<div class="title fx-grow">Add Content</div>
				<div class="tip fx fx--ac">ALT + N</div>
			</button>
		{/if}
		{#if entry}
			<button class="context-item fx" onclick={() => handleMoveEntries(entry)}>
				<div class="icon fx fx--cc">
					<iconify-icon icon="lucide:folder-input"></iconify-icon>
				</div>
				<div class="title fx-grow">Move To..</div>
				<div class="tip fx fx--ac"></div>
			</button>
			{#if isDir}
				<button class="context-item fx" disabled>
					<div class="icon fx fx--cc">
						<iconify-icon icon="lucide:download"></iconify-icon>
					</div>
					<div class="title fx-grow">Download {typeName}</div>
					<div class="tip fx fx--ac">D</div>
				</button>
			{:else}
				<button class="context-item fx" onclick={handleDownload}>
					<div class="icon fx fx--cc">
						<iconify-icon icon="lucide:download"></iconify-icon>
					</div>
					<div class="title fx-grow">Download {typeName}</div>
					<div class="tip fx fx--ac">D</div>
				</button>
			{/if}
			<button class="context-item fx" onclick={() => handleRenameEntry(entry)}>
				<div class="icon fx fx--cc">
					<iconify-icon icon="lucide:pencil-line"></iconify-icon>
				</div>
				<div class="title fx-grow">Rename {typeName}</div>
				<div class="tip fx fx--ac">ALT + R</div>
			</button>
			<button class="context-item fx" onclick={() => handleDeleteEntries([entry])}>
				<div class="icon fx fx--cc">
					<iconify-icon icon="lucide:trash-2"></iconify-icon>
				</div>
				<div class="title fx-grow">Delete {typeName}</div>
				<div class="tip fx fx--ac">DEL</div>
			</button>
		{/if}
	</section>
</div>

<style lang="scss">
	.context-menu {
		position: absolute;
		background: var(--clr-background-1);
		color: var(--clr-text-1);
		border-width: 2px;
		padding: 0;
		outline: none;

		.context-menu-container {
			> .context-item {
				cursor: pointer;
				padding: 0.5rem;
				text-decoration: none;
				color: inherit;
				// border-bottom: 1px solid var(--clr-border);

				// &:last-of-type {
				// 	border-bottom: none;
				// }

				&:hover:not(:disabled) {
					background: var(--clr-background);
				}

				&:disabled {
					opacity: 0.4;
					cursor: not-allowed;
				}

				> .icon {
					width: 1.5rem;
				}

				> .title {
					margin: 0 1rem;
				}

				> .tip {
					width: 4rem;
					color: var(--clr-text-2);
					opacity: 0.5; // temp
					font-size: 0.8em;
				}
			}
		}
	}
</style>
