<script module lang="ts">
	import type { Entry } from './entry';

	export type ContextMenuProps = {
		dialog: HTMLElement;
		position: { x: number; y: number };
		entry?: Entry;
		space?: string;
		href_base?: string;
		onDownload?: (entry: Entry) => void;
		handleAddContent?: (entries: (Entry | undefined)[]) => void;
		handleMoveEntries?: (entry: Entry) => void;
		handleDeleteEntries?: (entries: Entry[]) => void;
		handleRenameEntry?: (entry: Entry) => void;
	};
</script>

<script lang="ts">
	import { join } from '$lib/fs';
	import { EntryType, SpecialType } from './entry';
	import IconLucidePlus from '~icons/lucide/plus';
	import IconLucideFolderOpen from '~icons/lucide/folder-open';
	import IconLucideFolderInput from '~icons/lucide/folder-input';
	import IconLucideDownload from '~icons/lucide/download';
	import IconLucidePencilLine from '~icons/lucide/pencil-line';
	import IconLucideTrash2 from '~icons/lucide/trash-2';
	import IconLucideLink from '~icons/lucide/link';
	import IconLucideExternalLink from '~icons/lucide/external-link';

	let {
		dialog = $bindable(),
		position,
		entry = $bindable(),
		space,
		href_base,
		onDownload,
		handleAddContent,
		handleMoveEntries,
		handleDeleteEntries,
		handleRenameEntry
	}: ContextMenuProps = $props();

	/*
	 * todo: the context menu should have contexual visual differences for when we're in different states
	 *   for example when we have things selected it should clearly organize actions that are targetting the selection
	 *   on the same thread we need to break up the context menu into different sections
	 */

	const isDir = $derived(entry?.entry_type === EntryType.DIRECTORY);
	const typeName = $derived(isDir ? 'Folder' : 'File');

	const open_href = $derived(
		href_base ? `${href_base}/${entry?.path}` : `/${join('files', space!, entry?.path ?? '')}`
	);

	const is_share = $derived(entry?.special_type === SpecialType.SHARE);
	const share_id = $derived(entry?.special_fields?.id);
	const share_href = $derived(share_id ? `/shares/${share_id}` : undefined);

	const copyShareLink = () => {
		if (!share_href) return;
		navigator.clipboard.writeText(`${location.origin}${share_href}`);
		dialog.hidePopover();
	};

	// todo: check that this isn't too expensive
	const handleWindowClick = (e: MouseEvent) => {
		if (e.button !== 0) return;
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
		{#if entry && isDir}
			<a class="context-item fx" href={open_href} onclick={() => dialog.hidePopover()}>
				<div class="icon fx fx--cc">
					<IconLucideFolderOpen />
				</div>
				<div class="title fx-grow">Open {typeName}</div>
				<div class="tip fx fx--ac"></div>
			</a>
		{/if}
		{#if is_share && share_href}
			<a
				class="context-item fx"
				href={share_href}
				target="_blank"
				rel="noopener noreferrer"
				onclick={() => dialog.hidePopover()}
			>
				<div class="icon fx fx--cc">
					<IconLucideExternalLink />
				</div>
				<div class="title fx-grow">Open Share</div>
				<div class="tip fx fx--ac"></div>
			</a>
			<button class="context-item fx" onclick={copyShareLink}>
				<div class="icon fx fx--cc">
					<IconLucideLink />
				</div>
				<div class="title fx-grow">Copy Share Link</div>
				<div class="tip fx fx--ac"></div>
			</button>
		{/if}
		{#if handleAddContent && (!entry || isDir)}
			<button class="context-item fx" onclick={() => handleAddContent?.([entry])}>
				<div class="icon fx fx--cc">
					<IconLucidePlus />
				</div>
				<div class="title fx-grow">Add Content</div>
				<div class="tip fx fx--ac">ALT + N</div>
			</button>
		{/if}
		{#if entry}
			{#if handleMoveEntries}
				<button class="context-item fx" onclick={() => handleMoveEntries?.(entry)}>
					<div class="icon fx fx--cc">
						<IconLucideFolderInput />
					</div>
					<div class="title fx-grow">Move To..</div>
					<div class="tip fx fx--ac"></div>
				</button>
			{/if}
			{#if onDownload}
				{#if isDir}
					<button class="context-item fx" disabled>
						<div class="icon fx fx--cc">
							<IconLucideDownload />
						</div>
						<div class="title fx-grow">Download {typeName}</div>
						<div class="tip fx fx--ac">D</div>
					</button>
				{:else}
					<button class="context-item fx" onclick={() => onDownload?.(entry)}>
						<div class="icon fx fx--cc">
							<IconLucideDownload />
						</div>
						<div class="title fx-grow">Download {typeName}</div>
						<div class="tip fx fx--ac">D</div>
					</button>
				{/if}
			{/if}
			{#if handleRenameEntry}
				<button class="context-item fx" onclick={() => handleRenameEntry?.(entry)}>
					<div class="icon fx fx--cc">
						<IconLucidePencilLine />
					</div>
					<div class="title fx-grow">Rename {typeName}</div>
					<div class="tip fx fx--ac">ALT + R</div>
				</button>
			{/if}
			{#if handleDeleteEntries}
				<button class="context-item fx" onclick={() => handleDeleteEntries?.([entry])}>
					<div class="icon fx fx--cc">
						<IconLucideTrash2 />
					</div>
					<div class="title fx-grow">Delete {typeName}</div>
					<div class="tip fx fx--ac">DEL</div>
				</button>
			{/if}
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

				&:hover:not(:disabled) {
					background: var(--clr-background);
				}

				&:disabled {
					opacity: 0.4;
					cursor: not-allowed;
				}

				> .icon {
					font-size: 0.9rem;
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
