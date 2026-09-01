<script lang="ts">
	let { loading = false }: { loading?: boolean } = $props();

	let started = $state(false);

	$effect(() => {
		if (loading) started = true;
	});
</script>

<div class="loading-bar" class:loading class:started role="presentation"></div>

<style lang="scss">
	.loading-bar {
		--scan-width: 40%;
		height: 1px;
		background-color: var(--clr-border);
		overflow: hidden;
		position: relative;
		container-type: inline-size;
	}

	.loading-bar::after {
		content: '';
		position: absolute;
		top: 0;
		bottom: 0;
		left: 0;
		width: var(--scan-width);
		background: linear-gradient(90deg, transparent, var(--clr-primary), transparent);
		transform: translateX(-100%);
		will-change: transform;
	}

	.loading-bar.loading::after {
		animation: loading-slide 1.1s ease-in-out infinite;
	}

	.loading-bar.started:not(.loading)::after {
		animation: loading-finish 1.1s ease-in-out forwards;
	}

	@media (prefers-reduced-motion: reduce) {
		.loading-bar::after {
			animation: none;
		}
	}

	@keyframes loading-slide {
		from {
			transform: translateX(-100%);
		}
		to {
			transform: translateX(100cqw);
		}
	}

	@keyframes loading-finish {
		from {
			transform: translateX(-100%);
		}
		to {
			transform: translateX(100cqw);
		}
	}
</style>
