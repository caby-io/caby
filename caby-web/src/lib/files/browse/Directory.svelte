<script lang="ts">
	import type { DirFields, EntryProps } from '$lib/files/entry';
	import { join } from '$lib/fs';
	import IconFlatColorIconsFolder from '~icons/flat-color-icons/folder';

	let {
		entry,
		space,
		href_base,
		selection_mode = false,
		onSelect,
		onDragStart,
		onDragEnd,
		onDragEnter,
		onDragOver,
		onDragLeave,
		onDrop,
		onContextMenu
	}: EntryProps<DirFields> = $props();

	let is_selected = $derived(entry.is_selected);
	let is_targetted = $derived(entry.is_targetted);
	let is_processing = $derived(entry.is_processing);

	let href = $derived(
		href_base ? `${href_base}/${entry.path}` : `/${join('files', space!, entry.path)}`
	);
</script>

<div
	role="none"
	draggable="true"
	class="entry entry--directory"
	class:is_selected
	class:is_targetted
	class:is_processing
	onclick={onSelect}
	ondragstart={(e) => onDragStart?.(e, entry)}
	ondragend={(e) => onDragEnd?.(e, entry)}
	ondragenter={(e) => onDragEnter?.(e, entry)}
	ondragover={(e) => onDragOver?.(e, entry)}
	ondragleave={(e) => onDragLeave?.(e, entry)}
	ondrop={(e) => onDrop?.(e, entry)}
	oncontextmenu={(e) => onContextMenu?.(e, entry)}
>
	<section class="display fx fx--cc fx-grow"><IconFlatColorIconsFolder /></section>
	<section class="info">
		<h1>
			<a
				onclick={(e) => {
					if (selection_mode) e.preventDefault();
					else onSelect = undefined;
				}}
				{href}>{entry.name}</a
			>
		</h1>
		{entry.pretty_modified_at}
	</section>
</div>

<style lang="scss">
	@use 'entry';
</style>
