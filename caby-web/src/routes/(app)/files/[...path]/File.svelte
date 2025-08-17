<script lang="ts">
	import { join } from '$lib/fs';

	export type FileEntry = {
		entry_type: string;
		name: string;
		path: string;
		created_at: string;
		pretty_created_at: string;
		modified_at: string;
		pretty_modified_at: string;
		entry_fields: {
			size: number;
			pretty_size: string;
		};

		selected: boolean;
	};

	let { entry, onDelete, onRename }: { entry: FileEntry; onDelete: any; onRename: any } = $props();
</script>

<div
	class="entry entry--file {entry.selected ? 'entry--selected' : ''}"
	role="none"
	draggable="true"
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
