<script lang="ts">
	import type { DirFields, EntryProps } from '$lib/files/entry';
	import { join } from '$lib/fs';

	let {
		entry,
		space,
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
</script>

<div
	role="none"
	draggable="true"
	class="entry entry--directory"
	class:is_selected
	class:is_targetted
	class:is_processing
	onclick={onSelect}
	ondragstart={(e) => onDragStart!(e, entry)}
	ondragend={(e) => onDragEnd!(e, entry)}
	ondragenter={(e) => onDragEnter!(e, entry)}
	ondragover={(e) => onDragOver!(e, entry)}
	ondragleave={(e) => onDragLeave!(e, entry)}
	ondrop={(e) => onDrop!(e, entry)}
	oncontextmenu={(e) => onContextMenu!(e, entry)}
>
	<section class="display fx fx--cc fx-grow">📁</section>
	<section class="info">
		<h1>
			<!-- todo: figure out a better solution for the double event -->
			<a onclick={() => (onSelect = undefined)} href={`/${join('files', space!, entry.path)}`}
				>{entry.name}</a
			>
		</h1>
		{entry.pretty_modified_at}
	</section>
</div>

<style lang="scss">
	@use 'entry';
</style>
