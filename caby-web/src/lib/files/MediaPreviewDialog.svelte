<script lang="ts">
	import IconLucideX from '~icons/lucide/x';
	import IconLucideChevronLeft from '~icons/lucide/chevron-left';
	import IconLucideChevronRight from '~icons/lucide/chevron-right';
	import { swipe } from '$lib/actions/swipe';
	import type { Entry } from './entry';

	let { entries }: { entries: Entry[] } = $props();

	let dialog: HTMLDialogElement;
	let dialog_open = $state(false);
	let current_index = $state(0);
	let current_entry = $derived(entries[current_index]);
	let kind = $derived(current_entry?.entry_fields.kind);
	let media_url = $derived(current_entry?.entry_fields.media_url);
	let has_prev = $derived(current_index > 0);
	let has_next = $derived(current_index < entries.length - 1);

	// Video/Audio controls
	let autoplay = $state(false);
	let muted = $state(true);

	const onDialogOpen = () => {
		const opened_kind = current_entry?.entry_fields.kind;
		autoplay = opened_kind === 'audio' || opened_kind === 'video';
		muted = opened_kind !== 'audio';
		dialog_open = true;
	};

	const onDialogClose = () => {
		dialog_open = false;
	};

	export function openDialog(index: number) {
		current_index = index;
		onDialogOpen();
		dialog.showModal();
	}

	const onMediaPlay = () => {
		autoplay = true;
	};

	// Preload neighbor media
	let preload_urls = $derived(
		[current_index - 1, current_index + 1]
			.map((i) => entries[i])
			.filter((e) => e?.entry_fields.kind === 'image' && !!e.entry_fields.media_url)
			.map((e) => e.entry_fields.media_url!)
	);

	// Generate URLs with tokens to be able to fetch content
	$effect(() => {
		preload_urls.forEach((url) => {
			const img = new Image();
			img.src = url;
		});
	});

	const prev = () => {
		if (has_prev) current_index--;
	};
	const next = () => {
		if (has_next) current_index++;
	};

	const onKeyDown = (e: KeyboardEvent) => {
		if (!dialog?.open) return;
		if (e.key === 'ArrowLeft') {
			e.preventDefault();
			prev();
		} else if (e.key === 'ArrowRight') {
			e.preventDefault();
			next();
		}
	};

	const onSwipe = (dir: 'left' | 'right') => {
		if (dir === 'left') next();
		else prev();
	};
</script>

<svelte:window onkeydown={onKeyDown} />

<dialog bind:this={dialog} class="preview-dialog border-0" onclose={onDialogClose}>
	{#if dialog_open && current_entry}
		<div class="content fx fx--col">
			<header class="fx fx--ac">
				<span class="name fx-grow" title={current_entry.name}>{current_entry.name}</span>
				<span class="counter">{current_index + 1} / {entries.length}</span>
				<button
					class="icon-btn fx fx--cc"
					aria-label="previous"
					disabled={!has_prev}
					onclick={prev}
				>
					<IconLucideChevronLeft />
				</button>
				<button class="icon-btn fx fx--cc" aria-label="next" disabled={!has_next} onclick={next}>
					<IconLucideChevronRight />
				</button>
				<button class="icon-btn fx fx--cc" aria-label="close" onclick={() => dialog.close()}>
					<IconLucideX />
				</button>
			</header>
			<main class="stage fx fx--cc fx-grow" use:swipe={{ on_swipe: onSwipe }}>
				{#if media_url}
					{#key media_url}
						{#if kind === 'image'}
							<img src={media_url} alt={current_entry.name} />
						{:else if kind === 'video'}
							<!-- svelte-ignore a11y_media_has_caption -->
							<video src={media_url} controls {autoplay} bind:muted onplay={onMediaPlay}></video>
						{:else if kind === 'audio'}
							<audio src={media_url} controls {autoplay} bind:muted onplay={onMediaPlay}></audio>
						{/if}
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
		outline: none;

		> .content {
			width: 100%;
			height: 100%;
		}

		header {
			padding: 0.75rem 1rem;
			gap: 0.75rem;
			color: var(--clr-text-1);

			> .name {
				min-width: 0;
				overflow: hidden;
				text-overflow: ellipsis;
				white-space: nowrap;
			}

			> .counter {
				flex-shrink: 0;
				white-space: nowrap;
				font-size: 0.9em;
				color: var(--clr-text-2);
			}

			> .icon-btn {
				background: transparent;
				color: inherit;
				cursor: pointer;
				font-size: 1.4rem;
				padding: 0.25rem;
				border: none;

				&:disabled {
					opacity: 0.3;
					cursor: not-allowed;
				}
			}
		}

		main.stage {
			min-height: 0;
			padding: 1rem;
			touch-action: none;

			img,
			video {
				max-width: 100%;
				max-height: 100%;
				object-fit: contain;
			}

			img {
				touch-action: none;
			}

			audio {
				width: min(100%, 32rem);
			}
		}
	}
</style>
