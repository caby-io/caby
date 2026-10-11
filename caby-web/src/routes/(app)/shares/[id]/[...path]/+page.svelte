<script lang="ts">
	import { page } from '$app/state';
	import { client } from '$lib/stores/client.svelte';
	import { useAppLoading } from '$lib/stores/loading.svelte';
	import {
		getShare,
		listShare,
		authSharePassword,
		getShareOverview,
		type ShareAuthOptions
	} from '$lib/api/api_shares';
	import { syncGuestToken, ensureGuestToken, clearGuestToken } from '$lib/api/guest';
	import { EntryType } from '$lib/files/entry';
	import type { Entry } from '$lib/files/entry';
	import type { SelectedEntry } from '$lib/files/select';
	import type { OverviewEntry } from '$lib/files/overview/overview_entry';
	import EntriesOverviewNav from '$lib/files/overview/EntriesOverviewNav.svelte';
	import SidebarSection from '$lib/sidebar/SidebarSection.svelte';
	import { expandToPath } from '$lib/stores/overview.svelte';
	import EntriesGrid from '$lib/files/browse/EntriesGrid.svelte';
	import Breadcrumbs from '$lib/files/browse/Breadcrumbs.svelte';
	import ContextMenu from '$lib/files/ContextMenu.svelte';
	import MediaPreviewDialog from '$lib/files/MediaPreviewDialog.svelte';
	import PasswordPanel from './PasswordPanel.svelte';
	import ShareInfo from './ShareInfo.svelte';
	import IconLucideShare2 from '~icons/lucide/share-2';
	import { useSidebarSection } from '$lib/sidebar/sidebar.svelte';

	const id = $derived(page.params.id!);
	const path = $derived(page.params.path ?? '');
	const href_base = $derived(`/shares/${id}`);

	let loading = $state(true);

	useAppLoading(() => loading);
	let error = $state<string | undefined>();
	let auth = $state<ShareAuthOptions>();
	let space = $state('');
	let root_name = $state('');

	let need_password = $state(false);
	let password_error = $state<string | undefined>();
	let password_loading = $state(false);

	let tree = $state<{ id: string; entries: OverviewEntry[] }>();

	let entries = $state<Entry[]>([]);
	let dir_entries = $derived(entries.filter((e) => e.entry_type === EntryType.DIRECTORY));
	let file_entries = $derived(entries.filter((e) => e.entry_type === EntryType.FILE));

	let in_selection = $state(false);
	let selected_entries = $derived(new Set(entries.filter((e) => e.is_selected === true)));
	let last_selected: SelectedEntry | undefined = $state();

	$effect(() => {
		if (in_selection && selected_entries.size === 0) in_selection = false;
	});

	const handleSelectOp = (e: MouseEvent, selected: SelectedEntry) => {
		in_selection = true;

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

	const tryLoadTree = async (i: string): Promise<void> => {
		if (tree?.id === i) return;
		const resp = await getShareOverview(client, i);
		if (resp.status === 'success' && resp.data) tree = { id: i, entries: resp.data.entries };
	};

	const applyList = (i: string, loaded: Entry[]) => {
		entries = loaded;
		need_password = false;
		error = undefined;
		tryLoadTree(i);
	};

	const tryList = async (i: string, p: string): Promise<void> => {
		const resp = await listShare(client, i, p);
		if (resp.status === 'success' && resp.data) {
			applyList(i, resp.data.entries);
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
					applyList(i, retry.data.entries);
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

	$effect(() => {
		expandToPath(href_base, path);
	});

	useSidebarSection({ id: 'share-info', slot: 'main', content: share_info });
</script>

{#snippet share_info()}
	{#if root_name}
		<ShareInfo {root_name} {id} />
	{/if}
	{#if tree?.id === id}
		<SidebarSection title="Files">
			<EntriesOverviewNav overview_entries={tree.entries} {href_base} current_path={path} />
		</SidebarSection>
	{/if}
{/snippet}

<div class="share-view fx">
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
				<Breadcrumbs {href_base} root_icon={IconLucideShare2} />
			</header>
			<EntriesGrid
				{dir_entries}
				{file_entries}
				{space}
				{href_base}
				{in_selection}
				dimmed={loading}
				{handleSelectOp}
				{handleContextMenu}
				{handlePreview}
			/>
			<ContextMenu
				bind:dialog={contextMenuDialog}
				position={contextMenuProps.position}
				bind:entry={contextMenuProps.entry}
				{href_base}
				onDownload={downloadShareEntry}
			/>
			<MediaPreviewDialog bind:this={preview} entries={preview_entries} />
		{/if}
	</section>
</div>

<style lang="scss">
	.right {
		min-height: calc(100vh - var(--top-nav-height) - 1px);
	}

	.status {
		color: var(--clr-text-2);
		padding: 2rem;
		text-align: center;
	}

	.gate {
		position: fixed;
		inset: 0;
		z-index: 3;
		padding: 1rem;
		overflow-y: auto;
		background-color: var(--clr-background);
	}

	.bar {
		background-color: var(--clr-background-1);
		border-bottom: 1px solid var(--clr-border);
		padding: 0.5rem 1rem;
	}
</style>
