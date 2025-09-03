<script lang="ts">
	import { join } from '$lib/fs';
	import type { EntryProps, FileFields } from './entry';

	let {
		entry,
		isSelected: is_selected = false,
		// temp
		onSelect = undefined,
		onDragStart = undefined,
		onDragEnd = undefined,
		onDragEnter = undefined,
		onDragOver = undefined,
		onDragLeave = undefined,
		onDrop = undefined
	}: EntryProps<FileFields> = $props();
</script>

<div
	class="entry entry--file {is_selected ? 'entry--selected' : ''}"
	role="none"
	draggable="true"
	onclick={onSelect}
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
