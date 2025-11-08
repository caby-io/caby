<script module>
	import type { Entry } from './entry';

	export type ContextMenuProps = {
		dialog: HTMLElement;
		position: { x: number; y: number };
		entry?: Entry;
	};
</script>

<script lang="ts">
	let { dialog = $bindable(), position, entry = $bindable() }: ContextMenuProps = $props();

	// todo: check that this isn't too expensive
	const handleWindowClick = (e: MouseEvent) => {
		if (dialog.contains(e.target as Node)) {
			return;
		}
		dialog.hidePopover();
	};

	const onbeforetoggle = (e: ToggleEvent) => {
		if (!entry) {
			return;
		}

		if (e.newState === 'open') {
			entry.is_targetted = true;
			return;
		}
		entry.is_targetted = false;
	};

	// const oncontextmenu = (e: MouseEvent) => {
	// 	e.preventDefault();
	// 	dialog.hidePopover();
	// };
</script>

<svelte:window onclick={handleWindowClick} />

<div
	bind:this={dialog}
	class="context-menu border-0 box-shadow-0-card"
	style="left: {position.x}px; top: {position.y}px"
	popover
	{onbeforetoggle}
>
	<div class="context-item fx">
		<div class="icon fx fx--cc">
			<iconify-icon icon="lucide:plus"></iconify-icon>
		</div>
		<div class="title fx-grow">Add New File</div>
		<div class="tip fx fx--ac">CTRL + N</div>
	</div>
	{#if entry}
		<div class="context-item fx">
			<div class="icon fx fx--cc">
				<iconify-icon icon="lucide:folder-input"></iconify-icon>
			</div>
			<div class="title fx-grow">Move To..</div>
			<div class="tip fx fx--ac"></div>
		</div>
		<div class="context-item fx">
			<div class="icon fx fx--cc">
				<iconify-icon icon="lucide:download"></iconify-icon>
			</div>
			<div class="title fx-grow">Download File</div>
			<div class="tip fx fx--ac">D</div>
		</div>
		<div class="context-item fx">
			<div class="icon fx fx--cc">
				<iconify-icon icon="lucide:trash-2"></iconify-icon>
			</div>
			<div class="title fx-grow">Delete File</div>
			<div class="tip fx fx--ac">DEL</div>
		</div>
	{/if}
</div>

<style lang="scss">
	.context-menu {
		position: absolute;
		background: var(--clr-background-1);
		color: var(--clr-text-1);
		border-width: 2px;
		padding: 0;
		outline: none;

		// &.open {
		// 	display: flex;
		// 	flex-direction: column;
		// }

		> .context-item {
			cursor: pointer;
			padding: 0.5rem;
			// border-bottom: 1px solid var(--clr-border);

			// &:last-of-type {
			// 	border-bottom: none;
			// }

			&:hover {
				background: var(--clr-background);
			}

			> .icon {
				width: 1.5rem;
			}

			> .title {
				margin: 0 1rem;
			}

			> .tip {
				width: 4rem;
				display: flex;
				color: var(--clr-text-2);
				opacity: 0.5; // temp
				font-size: 0.8em;
			}
		}
	}
</style>
