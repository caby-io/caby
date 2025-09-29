<script lang="ts">
	import type { DirFields, EntryProps } from '$lib/files/entry';
	import { join } from '$lib/fs';

	let {
		entry,
		onSelect,
		onDragStart,
		onDragEnd,
		onDragEnter,
		onDragOver,
		onDragLeave,
		onDrop
	}: EntryProps<DirFields> = $props();

	let is_selected = $derived(entry.is_selected);
</script>

<div
	class="entry entry--directory"
	role="none"
	draggable="true"
	class:is_selected
	onclick={onSelect}
	ondragstart={(e) => onDragStart!(e, entry)}
	ondragend={(e) => onDragEnd!(e, entry)}
	ondragenter={(e) => onDragEnter!(e, entry)}
	ondragover={(e) => onDragOver!(e, entry)}
	ondragleave={(e) => onDragLeave!(e, entry)}
	ondrop={(e) => onDrop!(e, entry)}
>
	<section class="display fx fx--cc fx-grow">📁</section>
	<section class="info">
		<h1>
			<!-- todo: figure out a better solution for the double event -->
			<a onclick={() => (onSelect = undefined)} href={`/${join('files', entry.path)}`}
				>{entry.name}</a
			>
		</h1>
		{entry.pretty_modified_at}
	</section>
</div>

<style lang="scss">
	@use 'entry';
</style>
