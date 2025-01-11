<script lang="ts">
	import { join } from '$lib/helpers';

	type FileEntry = {
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
	};

	let { onDelete, file_entry }: { onDelete: any; file_entry: FileEntry } = $props();

	// export let file_entry: FileEntry;

	const delete_file = () => {};
</script>

<tr>
	<td data-cell="select" class="check"
		><iconify-icon icon="lucide:square-check-big"></iconify-icon></td
	>
	<td data-cell="main" class="main">
		<div class="fx fx-cc">
			<div class="icon fx fx-cc">📃</div>
			<div class="text fx-grow">
				<div class="name">{file_entry.name}</div>
				<div class="size">{file_entry.entry_fields.pretty_size}</div>
			</div>
		</div>
	</td>
	<td data-cell="actions" class="actions">
		<div class="fx fx-ac">
			<a
				class="action fx fx-cc"
				href={'http://localhost:8080/v0' + join('download', file_entry.path!)}
				download={file_entry.name}
			>
				<iconify-icon icon="lucide:hard-drive-download"></iconify-icon>
			</a>
			<div class="action fx fx-cc">
				<iconify-icon icon="lucide:info"></iconify-icon>
			</div>
			<div class="action fx fx-cc" on:click={() => onDelete(file_entry.path)}>
				<iconify-icon icon="lucide:trash-2"></iconify-icon>
			</div>
			<div class="action fx fx-cc">
				<iconify-icon icon="lucide:more-horizontal"></iconify-icon>
			</div>
		</div>
	</td>
	<td data-cell="last-modified">{file_entry.pretty_modified_at}</td>
</tr>

<style lang="scss">
	@import 'entry';
</style>
