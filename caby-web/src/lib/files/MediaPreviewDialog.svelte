<script lang="ts">
	import IconLucideX from '~icons/lucide/x';
	import type { Entry } from './entry';

	let {
		open = $bindable(),
		entries,
		start_index
	}: {
		open: boolean;
		entries: Entry[];
		start_index: number;
	} = $props();

	let dialog: HTMLDialogElement;
	let current_index = $state(0);
	let current_entry = $derived(entries[current_index]);
	let media_url = $derived(current_entry?.entry_fields.media_url);

	$effect(() => {
		if (open && dialog && !dialog.open) {
			current_index = start_index;
			dialog.showModal();
		} else if (!open && dialog?.open) {
			dialog.close();
		}
	});

	const syncOpenOnClose = () => {
		open = false;
	};
</script>

<dialog bind:this={dialog} class="preview-dialog border-0" onclose={syncOpenOnClose}>
	{#if current_entry}
		<div class="content fx fx--col">
			<header class="fx fx--ac">
				<span class="name fx-grow" title={current_entry.name}>{current_entry.name}</span>
				<button class="close fx fx--cc" aria-label="close" onclick={() => dialog.close()}>
					<IconLucideX />
				</button>
			</header>
			<main class="stage fx fx--cc fx-grow">
				{#if media_url}
					{#key media_url}
						<img src={media_url} alt={current_entry.name} />
					{/key}
				{/if}
			</main>
		</div>
	{/if}
</dialog>

<style lang="scss">
	.preview-dialog {
		position: fixed;
		inset: 0;
		width: 100vw;
		height: 100vh;
		max-width: 100vw;
		max-height: 100vh;
		padding: 0;
		background: rgba(0, 0, 0, 0.92);
		color: var(--clr-text-0);

		> .content {
			width: 100%;
			height: 100%;
		}

		header {
			padding: 0.75rem 1rem;
			gap: 1rem;
			color: var(--clr-text-1);

			> .name {
				overflow: hidden;
				text-overflow: ellipsis;
				white-space: nowrap;
			}

			> .close {
				background: transparent;
				color: inherit;
				cursor: pointer;
				font-size: 1.4rem;
				padding: 0.25rem;
				border: none;
			}
		}

		main.stage {
			min-height: 0;
			padding: 1rem;

			img {
				max-width: 100%;
				max-height: 100%;
				object-fit: contain;
			}
		}
	}
</style>
