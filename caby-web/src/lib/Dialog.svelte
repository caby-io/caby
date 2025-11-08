<script lang="ts">
	import type { Snippet } from 'svelte';

	// let dialog: HTMLDialogElement;

	// let { openEmitter } = $props();
	let {
		dialog = $bindable(),
		title,
		content
	}: { dialog: HTMLDialogElement; title?: string; content: Snippet } = $props();

	// close the dialog if a click is detected outside
	const onclick = (e: Event) => {
		if (e.target === e.currentTarget) {
			dialog.close();
		}
	};

	// $effect(() => {
	// 	openEmitter.setEvent(() => value.showModal());
	// });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<dialog bind:this={dialog} class="dialog border-0 box-shadow-0-card" {onclick}>
	{#if title}
		<header class="title fx fx--ac">
			<h3 class="fx-grow">{title}</h3>
			<button class="fx fx--cc" aria-label="close" onclick={() => dialog.close()}>
				<iconify-icon icon="lucide:x"></iconify-icon>
			</button>
		</header>
	{/if}
	{@render content()}
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

		> header.title {
			background: var(--clr-background-2);
			padding: 1rem;
			border-bottom: 1px solid var(--clr-border);

			> button {
				font-size: 1.2rem;
				cursor: pointer;
			}
		}
	}
</style>
