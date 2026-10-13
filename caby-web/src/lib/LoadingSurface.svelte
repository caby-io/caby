<script lang="ts">
	import { fade } from 'svelte/transition';

	let { loading = false, progress }: { loading?: boolean; progress?: number } = $props();

	let clamped = $derived(progress === undefined ? undefined : Math.min(Math.max(progress, 0), 1));

	const syncShimmer = (event: AnimationEvent & { currentTarget: HTMLDivElement }) => {
		for (const animation of event.currentTarget.getAnimations()) {
			animation.startTime = 0;
		}
	};
</script>

{#if loading || clamped !== undefined}
	<div
		class="loading-surface"
		aria-hidden="true"
		out:fade={{ duration: 300 }}
		onoutrostart={(event) => event.currentTarget.classList.add('leaving')}
	>
		{#if loading}
			<div class="shimmer" onanimationstart={syncShimmer}></div>
		{/if}
		{#if clamped !== undefined}
			<div class="progress" style:--progress={clamped}></div>
		{/if}
	</div>
{/if}

<style lang="scss">
	.loading-surface {
		position: absolute;
		inset: 0;
		overflow: hidden;
		pointer-events: none;
	}

	.shimmer {
		--sheen: var(--clr-text-0);

		position: absolute;
		inset: 0 auto 0 0;
		width: 100%;
		background: linear-gradient(
			in oklab 90deg,
			transparent 0%,
			color-mix(in oklab, var(--sheen) 1.5%, transparent) 20%,
			color-mix(in oklab, var(--sheen) 4.5%, transparent) 35%,
			color-mix(in oklab, var(--sheen) 7%, transparent) 50%,
			color-mix(in oklab, var(--sheen) 4.5%, transparent) 65%,
			color-mix(in oklab, var(--sheen) 1.5%, transparent) 80%,
			transparent 100%
		);
		opacity: 0;
		transform: translateX(-100%);
		will-change: transform, opacity;
		animation: shimmer-sweep 2s infinite;
	}

	.progress {
		position: absolute;
		inset: 0 auto 0 0;
		width: calc(var(--progress) * 100%);
		background-color: color-mix(in oklab, var(--clr-accent) 40%, transparent);
		transition: width 0.5s linear;
	}

	.loading-surface:global(.leaving) .progress {
		width: 100%;
	}

	@media (prefers-reduced-motion: reduce) {
		.shimmer {
			animation: none;
		}

		.progress {
			transition: none;
		}
	}

	@keyframes shimmer-sweep {
		0% {
			opacity: 0;
			transform: translateX(-100%);
			animation-timing-function: cubic-bezier(0.45, 0, 0.25, 1);
		}
		15% {
			opacity: 1;
		}
		40% {
			opacity: 1;
		}
		55% {
			opacity: 0;
			transform: translateX(100%);
		}
		100% {
			opacity: 0;
			transform: translateX(100%);
		}
	}
</style>
