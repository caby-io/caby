<script lang="ts">
	import 'iconify-icon';

	let addEntryDialog: HTMLDialogElement;

	let openAddDialog = () => {
		addEntryDialog.showModal();
		// addEntryDialog.hideM
	};

	// let dialog: HTMLDialogElement | null = document.querySelector('#rename-modal');
	// dialog!.showModal();
</script>

<button class="add-button fx fx--cc" onclick={openAddDialog}>
	<iconify-icon icon="lucide:plus"></iconify-icon>
</button>

<dialog bind:this={addEntryDialog} id="add-entry-dialog">test</dialog>

<style lang="scss">
	.add-button {
		cursor: pointer;
		background-color: var(--clr-primary);
		color: var(--clr-background);
		font-size: 1.6rem;
		width: 2.2rem;
		height: 2.2rem;
		border-radius: 50%;
	}

	#add-entry-dialog {
		position: fixed;
		/* Move it 50% relative to container */
		top: 50%;
		left: 50%;
		/* Move it back 50% relative to self */
		-webkit-transform: translateX(-50%) translateY(-50%);
		-moz-transform: translateX(-50%) translateY(-50%);
		-ms-transform: translateX(-50%) translateY(-50%);
		transform: translateX(-50%) translateY(-50%);

		&::backdrop {
			background: rgba(0, 0, 0, 0.4);
			backdrop-filter: blur(1px);
			// transition: backdrop-filter 0.3s;
		}
	}
</style>
