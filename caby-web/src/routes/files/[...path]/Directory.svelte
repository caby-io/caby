<script lang="ts">
	import { join } from '$lib/fs';
	import { MoveOp } from './+page.svelte';

	export type DirEntry = {
		entry_type: string;
		name: string;
		path: string;
		created_at: string;
		pretty_created_at: string;
		modified_at: string;
		pretty_modified_at: string;
	};

	let {
		entry,
		onDelete,
		onRename,
		handleMoveOp
	}: { entry: DirEntry; onDelete: any; onRename: any; handleMoveOp: any } = $props();

	// Dragged Item
	let dragging = $state(false);

	const onDragStart = (e: DragEvent) => {
		dragging = true;
		handleMoveOp(MoveOp.ADD_SRC, entry);
	};

	const onDragEnd = (e: DragEvent) => {
		dragging = false;
		handleMoveOp(MoveOp.REM_SRC, entry);
	};

	// Dragged-To Item
	let dragoverCt = $state(0);
	let dragover = $derived(dragoverCt > 0);

	const onDragEnter = (e: DragEvent) => {
		if (dragging) {
			return false;
		}

		dragoverCt += 1;
		// if (dragover) {
		// 	handleMoveOp(MoveOp.SET_DST, entry);
		// }
	};

	const onDragOver = (e: DragEvent) => {
		if (dragging) {
			return false;
		}
		e.preventDefault();
	};

	const onDragLeave = (e: DragEvent) => {
		if (dragging) {
			return false;
		}

		dragoverCt -= 1;
		// if (!dragover) {
		// 	handleMoveOp(MoveOp.REM_DST, entry);
		// }
	};

	const onDrop = (e: DragEvent) => {
		handleMoveOp(MoveOp.EXEC, entry);
		dragoverCt = 0;
		// todo: bubble the event up
	};
</script>

<tr
	draggable="true"
	class:dragging
	class:dragover
	ondragstart={onDragStart}
	ondragend={onDragEnd}
	ondragenter={onDragEnter}
	ondragover={onDragOver}
	ondragleave={onDragLeave}
	ondrop={onDrop}
>
	<td data-cell="select" class="check"><iconify-icon icon="lucide:square"></iconify-icon></td>
	<!-- todo: improve -->
	<td data-cell="main" class="main">
		<div class="fx fx-cc">
			<div class="icon fx fx-cc"><a href={`/${join('files', entry.path)}`}>📁</a></div>
			<div class="text fx-grow">
				<div class="name"><a href={`/${join('files', entry.path)}`}>{entry.name}/</a></div>
				<div class="size">–</div>
			</div>
		</div>
	</td>
	<td data-cell="actions" class="actions">
		<div class="fx fx-ac">
			<div class="action fx fx-cc">
				<iconify-icon icon="lucide:hard-drive-download"></iconify-icon>
			</div>
			<div class="action fx fx-cc" onclick={() => onRename(entry)}>
				<iconify-icon icon="lucide:pencil"></iconify-icon>
			</div>
			<div class="action fx fx-cc" onclick={() => onDelete(entry.path)}>
				<iconify-icon icon="lucide:trash-2"></iconify-icon>
			</div>
			<div class="action fx fx-cc">
				<iconify-icon icon="lucide:more-horizontal"></iconify-icon>
			</div>
		</div>
	</td>
	<td data-cell="last-modified">{entry.pretty_modified_at}</td>
</tr>

<style lang="scss">
	@use 'entry';

	tr.dragging {
		background: red;
	}

	tr.dragover {
		background: yellow;
	}
</style>
