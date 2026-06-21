<script lang="ts">
	import type { EntryProps, FileFields } from '$lib/files/entry';
	import IconFlatColorIconsFile from '~icons/flat-color-icons/file';
	import IconFlatColorIconsImageFile from '~icons/flat-color-icons/image-file';
	import IconFlatColorIconsVideoFile from '~icons/flat-color-icons/video-file';
	import IconFlatColorIconsAudioFile from '~icons/flat-color-icons/audio-file';
	import IconFlatColorIconsDocument from '~icons/flat-color-icons/document';
	import IconFlatColorIconsPackage from '~icons/flat-color-icons/package';

	let {
		entry,
		selection_mode = false,
		onSelect,
		onPreview,
		onDragStart,
		onDragEnd,
		// onDragEnter,
		onDragOver,
		// onDragLeave,
		onDrop,
		onContextMenu
	}: EntryProps<FileFields> = $props();

	function pickKindIcon(kind: string) {
		switch (kind) {
			case 'image':
				return IconFlatColorIconsImageFile;
			case 'video':
				return IconFlatColorIconsVideoFile;
			case 'audio':
				return IconFlatColorIconsAudioFile;
			case 'pdf':
			case 'document':
				return IconFlatColorIconsDocument;
			case 'archive':
				return IconFlatColorIconsPackage;
			default:
				return IconFlatColorIconsFile;
		}
	}

	let is_selected = $derived(entry.is_selected);
	let is_processing = $derived(entry.is_processing);
	let is_targetted = $derived(entry.is_targetted);

	let img_failed = $state(false);
	let kind = $derived(entry.entry_fields.kind);
	let preview_url = $derived(entry.entry_fields.preview_url);
	let show_preview = $derived(kind === 'image' && !!preview_url && !img_failed);
	let KindIcon = $derived(pickKindIcon(kind));
	let can_preview = $derived(entry.entry_fields.can_preview);

	function handleDisplayClick(ev: MouseEvent) {
		ev.stopPropagation();
		if (!can_preview || selection_mode || ev.metaKey || ev.ctrlKey) {
			onSelect?.(ev);
		} else {
			onPreview?.(entry);
		}
	}
</script>

<div
	role="none"
	draggable="true"
	class="entry entry--file"
	class:is_selected
	class:is_targetted
	class:is_processing
	onclick={onSelect}
	ondragstart={(e) => onDragStart!(e, entry)}
	ondragend={(e) => onDragEnd!(e, entry)}
	ondragover={(e) => onDragOver!(e, entry)}
	ondrop={(e) => onDrop!(e, entry)}
	oncontextmenu={(e) => onContextMenu!(e, entry)}
>
	{#if show_preview}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<section class="display fx fx--cc fx-grow" class:can-preview={can_preview} onclick={handleDisplayClick}>
			<img src={preview_url} alt={entry.name} loading="lazy" onerror={() => (img_failed = true)} />
		</section>
	{:else}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<section class="display fx fx--cc fx-grow" class:can-preview={can_preview} onclick={handleDisplayClick}>
			<KindIcon />
		</section>
	{/if}
	<section class="info">
		<!-- todo: consider splitting extension so we can show it-->
		<h1 title={entry.name}>{entry.name}</h1>
		{entry.pretty_modified_at}
	</section>
</div>

<style lang="scss">
	@use 'entry';
</style>
