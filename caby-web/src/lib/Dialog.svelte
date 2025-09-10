<script lang="ts">
	let dialog: HTMLDialogElement;

	const onClick = (e: Event) => {
		if (e.target === e.currentTarget) {
			dialog.close();
		}
	};
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<dialog bind:this={dialog} class="dialog border-0 box-shadow-0-card" onclick={onClick}>
	<slot />
</dialog>

<style lang="scss">
	.dialog {
		position: fixed;
		left: 50%;
		top: 50%;
		transform: translate(-50%, -50%);
		-webkit-transform: translateX(-50%) translateY(-50%);
		-moz-transform: translateX(-50%) translateY(-50%);
		-ms-transform: translateX(-50%) translateY(-50%);
		padding: 0;
		background: var(--clr-background-1);
		color: var(--clr-text-1);
		width: clamp(30rem, 56%, 40rem);

		&::backdrop {
			background: rgba(0, 0, 0, 0.5);
			backdrop-filter: blur(2px);
			// transition: backdrop-filter 0.3s;
		}
	}
</style>
