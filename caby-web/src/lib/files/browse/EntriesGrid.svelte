<script lang="ts">
	import type { Entry } from '$lib/files/entry';
	import type { SelectedEntry } from '$lib/files/select';
	import Directory from './Directory.svelte';
	import File from './File.svelte';

	let {
		dir_entries,
		file_entries,
		space,
		href_base,
		in_selection = false,
		dimmed = false,
		drag_over_ct = 0,
		onDragEnter,
		onDragOver,
		onDragLeave,
		onDragEnd,
		onDrop,
		handleContextMenu,
		handleSelectOp,
		handlePreview,
		onEntryDragStart,
		onEntryDragEnd,
		onEntryDragEnter,
		onEntryDragOver,
		onEntryDragLeave,
		onEntryDrop
	}: {
		dir_entries: Entry[];
		file_entries: Entry[];
		space: string;
		href_base?: string;
		in_selection?: boolean;
		dimmed?: boolean;
		drag_over_ct?: number;
		onDragEnter?: (e: DragEvent) => void;
		onDragOver?: (e: DragEvent) => void;
		onDragLeave?: (e: DragEvent) => void;
		onDragEnd?: (e: DragEvent) => void;
		onDrop?: (e: DragEvent) => void;
		handleContextMenu?: (e: MouseEvent, entry?: Entry) => void;
		handleSelectOp?: (e: MouseEvent, selected: SelectedEntry) => void;
		handlePreview?: (entry: Entry) => void;
		onEntryDragStart?: (e: DragEvent, entry: Entry) => void;
		onEntryDragEnd?: (e: DragEvent, entry: Entry) => void;
		onEntryDragEnter?: (e: DragEvent, entry: Entry) => void;
		onEntryDragOver?: (e: DragEvent, entry: Entry) => void;
		onEntryDragLeave?: (e: DragEvent, entry: Entry) => void;
		onEntryDrop?: (e: DragEvent, entry: Entry) => void;
	} = $props();
</script>

<main
	class="entries fx-grow"
	class:drag-over={drag_over_ct > 0}
	class:dimmed
	ondragenter={onDragEnter}
	ondragover={onDragOver}
	ondragleave={onDragLeave}
	ondragend={onDragEnd}
	ondrop={onDrop}
	oncontextmenu={(e) => handleContextMenu?.(e)}
>
	<section class="directories">
		<h3>Directories</h3>
		<div class="dir-list">
			{#each dir_entries as entry, index}
				<Directory
					{entry}
					{space}
					{href_base}
					selection_mode={in_selection}
					onSelect={(e: MouseEvent) => handleSelectOp?.(e, { index, entry })}
					onDragStart={onEntryDragStart}
					onDragEnd={onEntryDragEnd}
					onDragEnter={onEntryDragEnter}
					onDragOver={onEntryDragOver}
					onDragLeave={onEntryDragLeave}
					onDrop={onEntryDrop}
					onContextMenu={handleContextMenu}
				/>
			{/each}
		</div>
	</section>
	<section class="files">
		<h3>Files</h3>
		<div class="file-list">
			{#each file_entries as entry, index}
				<File
					{entry}
					{space}
					selection_mode={in_selection}
					onSelect={(e: MouseEvent) =>
						handleSelectOp?.(e, { index: index + dir_entries.length, entry })}
					onPreview={handlePreview}
					onDragStart={onEntryDragStart}
					onDragEnd={onEntryDragEnd}
					onDragOver={onEntryDragOver}
					onDrop={onEntryDrop}
					onContextMenu={handleContextMenu}
				/>
			{/each}
		</div>
	</section>
</main>

<style lang="scss">
	main.entries {
		background: var(--clr-background);
		padding: 1rem;
		position: relative;
		transition: opacity 0.3s;

		&.dimmed {
			opacity: 0.6;
		}

		&.drag-over {
			opacity: 0.5;

			&::after {
				position: absolute;
				top: 0;
				left: 0;
				content: '';
				width: 100%;
				height: 100%;
				background: var(--clr-background);
				opacity: 0.4;
			}
		}
	}

	.directories,
	.files {
		> h3 {
			margin: 1rem 0;
		}

		&:first-of-type {
			> h3 {
				margin-top: 0;
			}
		}
	}

	.dir-list,
	.file-list {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(12rem, 1fr));
		grid-auto-rows: minmax(12rem, 1fr);
		gap: 0.75rem;
	}

	.file-list {
		padding-bottom: 8rem;
	}
</style>
