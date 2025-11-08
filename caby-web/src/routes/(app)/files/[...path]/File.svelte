<script lang="ts">
	import { join } from '$lib/fs';
	import type { EntryProps, FileFields } from '$lib/files/entry';

	let {
		entry,
		onSelect,
		onDragStart,
		onDragEnd,
		// onDragEnter,
		onDragOver,
		// onDragLeave,
		onDrop,
		onContextMenu
	}: EntryProps<FileFields> = $props();

	let is_selected = $derived(entry.is_selected);
	let is_processing = $derived(entry.is_processing);
	let is_targetted = $derived(entry.is_targetted);
	// let dragging = $state(false);

	// const handleDragStart = (e: Event) => {

	// }
</script>

<div
	class="entry entry--file"
	class:is_selected
	class:is_targetted
	class:is_processing
	role="none"
	draggable="true"
	onclick={onSelect}
	ondragstart={(e) => onDragStart!(e, entry)}
	ondragend={(e) => onDragEnd!(e, entry)}
	ondragover={(e) => onDragOver!(e, entry)}
	ondrop={(e) => onDrop!(e, entry)}
	oncontextmenu={(e) => onContextMenu!(e, entry)}
>
	<section class="display fx fx--cc fx-grow">📃</section>
	<section class="info">
		<!-- todo: consider splitting extension so we can show it-->
		<h1 title={entry.name}>{entry.name}</h1>
		{entry.pretty_modified_at}
	</section>
</div>

<style lang="scss">
	@use 'entry';
</style>
