<script lang="ts">
	import { page } from '$app/state';
	import { getContext } from 'svelte';
	import { client } from '$lib/stores/client.svelte';
	import {
		getShare,
		listShare,
		authSharePassword,
		type ShareAuthOptions
	} from '$lib/api/api_shares';
	import { syncGuestToken, ensureGuestToken, clearGuestToken } from '$lib/api/guest';
	import type { Entry } from '$lib/files/entry';
	import type { SelectedEntry } from '$lib/files/select';
	import EntriesGrid from '$lib/files/browse/EntriesGrid.svelte';
	import Breadcrumbs from '$lib/files/browse/Breadcrumbs.svelte';
	import ContextMenu from '$lib/files/ContextMenu.svelte';
	import MediaPreviewDialog from '$lib/files/MediaPreviewDialog.svelte';
	import PasswordPanel from './PasswordPanel.svelte';
	import ShareInfo from './ShareInfo.svelte';

	const id = $derived(page.params.id!);
	const path = $derived(page.params.path ?? '');
	const href_base = $derived(`/shares/${id}`);

	const menu = getContext<{ open: boolean }>('menu');

	let loading = $state(true);
	let error = $state<string | undefined>();
	let auth = $state<ShareAuthOptions>();
	let space = $state('');
	let root_name = $state('');

	let need_password = $state(false);
	let password_error = $state<string | undefined>();
	let password_loading = $state(false);

	let entries = $state<Entry[]>([]);
	let dir_entries = $derived(entries.filter((e) => e.entry_type === 'directory'));
	let file_entries = $derived(entries.filter((e) => e.entry_type === 'file'));

	let selection_mode = $state<null | 'touch' | 'desktop'>(null);
	let in_selection = $derived(selection_mode !== null);
	let selected_entries = $derived(new Set(entries.filter((e) => e.is_selected === true)));
	let last_selected: SelectedEntry | undefined = $state();

	$effect(() => {
		if (selection_mode === 'desktop' && selected_entries.size === 0) selection_mode = null;
	});

	const handleSelectOp = (e: MouseEvent, selected: SelectedEntry) => {
		if (selection_mode === null) selection_mode = 'desktop';

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

		selected.entry.is_selected = !selected.entry.is_selected;
		last_selected = selected;
	};

	let preview_entries = $derived(file_entries.filter((e) => e.entry_fields?.can_preview));
	// svelte-ignore non_reactive_update
	let preview: MediaPreviewDialog;

	const handlePreview = (entry: Entry) => {
		const index = preview_entries.indexOf(entry);
		if (index < 0) return;
		preview.openDialog(index);
	};

	// svelte-ignore non_reactive_update
	let contextMenuDialog: HTMLDialogElement;
	let contextMenuProps = $state<{ position: { x: number; y: number }; entry?: Entry }>({
		position: { x: 0, y: 0 },
		entry: undefined
	});

	const handleContextMenu = (e: MouseEvent, entry?: Entry) => {
		e.preventDefault();
		e.stopPropagation();

		const offset = 264;
		let x = e.pageX;
		let y = e.pageY;
		if (e.view && e.view.innerWidth - x < offset) x = e.view.innerWidth - offset;

		contextMenuProps = { position: { x, y }, entry };
		if (entry) entry.is_targetted = true;
		contextMenuDialog.showPopover();
	};

	const downloadShareEntry = (entry: Entry) => {
		const url = entry.entry_fields?.download_url;
		if (!url) return;
		const a = document.createElement('a');
		a.href = url;
		a.download = '';
		document.body.appendChild(a);
		a.click();
		a.remove();
	};

	const applyList = (loaded: Entry[]) => {
		entries = loaded;
		need_password = false;
		error = undefined;
	};

	const tryList = async (i: string, p: string): Promise<void> => {
		const resp = await listShare(client, i, p);
		if (resp.status === 'success' && resp.data) {
			applyList(resp.data.entries);
			return;
		}

		switch (resp.status_code) {
			case 403:
				if (auth?.password) need_password = true;
				else error = 'You do not have access to this share.';
				return;
			case 404:
				error = 'This share does not exist.';
				return;
			case 410:
				error = 'This share has expired.';
				return;
			case 401: {
				await clearGuestToken(client);
				const retry = await listShare(client, i, p);
				if (retry.status === 'success' && retry.data) {
					applyList(retry.data.entries);
				} else if (retry.status_code === 403 && auth?.password) {
					need_password = true;
				} else {
					error = 'Could not access this share.';
				}
				return;
			}
			default:
				error = 'Could not load this share.';
		}
	};

	const load = async (): Promise<void> => {
		const i = id;
		const p = path;

		loading = true;
		error = undefined;
		need_password = false;

		await syncGuestToken(client);

		const resp = await getShare(client, i);
		if (resp.status !== 'success' || !resp.data) {
			if (resp.status_code === 404) error = 'This share does not exist.';
			else if (resp.status_code === 410) error = 'This share has expired.';
			else error = 'Could not load this share.';
			loading = false;
			return;
		}

		auth = resp.data.auth;
		space = resp.data.space;
		root_name = resp.data.root_name;

		await tryList(i, p);
		loading = false;
	};

	const trySubmitPassword = async (submitted: string): Promise<void> => {
		password_loading = true;
		password_error = undefined;

		await ensureGuestToken(client);
		const resp = await authSharePassword(client, id, submitted);
		if (resp.status === 'success') {
			await tryList(id, path);
			password_loading = false;
			return;
		}

		if (resp.status_code === 401) password_error = 'Incorrect password.';
		else if (resp.status_code === 403) password_error = 'This share has no password access.';
		else password_error = 'Could not verify the password.';
		password_loading = false;
	};

	$effect(() => {
		load();
	});
</script>

<div class="share-view fx">
	<section class="left fx fx--col" class:open={menu.open}>
		{#if root_name}
			<ShareInfo {root_name} {id} />
		{/if}
	</section>
	<div
		class="menu-backdrop"
		class:open={menu.open}
		role="presentation"
		onclick={() => (menu.open = false)}
	></div>
	<section class="right fx-grow fx fx--col">
		{#if loading}
			<div class="status fx fx--cc fx-grow">Loading…</div>
		{:else if error}
			<div class="status fx fx--cc fx-grow">{error}</div>
		{:else if need_password}
			<div class="gate fx fx--cc fx-grow">
				<PasswordPanel
					onSubmit={trySubmitPassword}
					error={password_error}
					loading={password_loading}
				/>
			</div>
		{:else}
			<header class="bar fx fx--ac">
				<Breadcrumbs {href_base} />
			</header>
			<EntriesGrid
				{dir_entries}
				{file_entries}
				{space}
				{href_base}
				{in_selection}
				{handleSelectOp}
				{handleContextMenu}
				{handlePreview}
			/>
			<ContextMenu
				bind:dialog={contextMenuDialog}
				position={contextMenuProps.position}
				bind:entry={contextMenuProps.entry}
				onDownload={downloadShareEntry}
			/>
			<MediaPreviewDialog bind:this={preview} entries={preview_entries} />
		{/if}
	</section>
</div>

<style lang="scss">
	@use '$lib/styles/breakpoints' as bp;

	.share-view {
		border-top: 1px solid var(--clr-border);
	}

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

	.status {
		color: var(--clr-text-2);
		padding: 2rem;
		text-align: center;
	}

	.gate {
		padding: 2rem;
	}

	.bar {
		background-color: var(--clr-background-1);
		border-bottom: 1px solid var(--clr-border);
		padding: 0.5rem 1rem;
	}
</style>
