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

		selected: boolean;
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

<div
	class="entry entry--directory"
	role="none"
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
	<section class="display fx fx--cc fx-grow">📁</section>
	<section class="info">
		<h1><a href={`/${join('files', entry.path)}`}>{entry.name}</a></h1>
		{entry.pretty_modified_at}
	</section>
</div>

<style lang="scss">
	@use 'entry';

	tr.dragging {
		background: red;
	}

	tr.dragover {
		background: yellow;
	}
</style>
