<script lang="ts">
	import { join } from '$lib/helpers';

	type DirEntry = {
		entry_type: string;
		name: string;
		path: string;
		created_at: string;
		pretty_created_at: string;
		modified_at: string;
		pretty_modified_at: string;
	};

	export let dir_entry: DirEntry;
</script>

<tr>
	<td data-cell="select" class="check"><iconify-icon icon="lucide:square"></iconify-icon></td>
	<!-- todo: improve -->
	<td data-cell="main" class="main fx">
		<div class="icon fx fx-cc"><a href={join('files', dir_entry.path)}>📁</a></div>
		<div class="text fx-grow">
			<div class="name"><a href={join('files', dir_entry.path)}>{dir_entry.name}/</a></div>
			<div class="size">–</div>
		</div>
	</td>
	<td data-cell="actions" class="actions">
		<div class="fx fx-ac">
			<div class="action fx fx-cc">
				<iconify-icon icon="lucide:hard-drive-download"></iconify-icon>
			</div>
			<div class="action fx fx-cc">
				<iconify-icon icon="lucide:info"></iconify-icon>
			</div>
			<div class="action fx fx-cc">
				<iconify-icon icon="lucide:trash-2"></iconify-icon>
			</div>
			<div class="action fx fx-cc">
				<iconify-icon icon="lucide:more-horizontal"></iconify-icon>
			</div>
		</div>
	</td>
	<td data-cell="last-modified">{dir_entry.pretty_modified_at}</td>
</tr>

<style lang="scss">
	@import 'entry';
</style>
