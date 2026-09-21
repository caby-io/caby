<script lang="ts">
	let { name, size = '2rem' }: { name?: string | null; size?: string } = $props();

	const trimmed = $derived((name ?? '').trim());
	const initial = $derived(trimmed ? trimmed[0].toUpperCase() : '');
	const hue = $derived(
		(([...trimmed].reduce((h, c) => (c.charCodeAt(0) + ((h << 5) - h)) | 0, 0) % 360) + 360) % 360
	);
</script>

<span
	class="avatar fx fx--cc"
	class:placeholder={!trimmed}
	style="--avatar-size:{size}; --avatar-hue:{hue}"
	aria-hidden="true"
>
	{initial}
</span>

<style lang="scss">
	.avatar {
		width: var(--avatar-size);
		height: var(--avatar-size);
		border-radius: 50%;
		background: hsl(var(--avatar-hue) 55% 45%);
		color: #fff;
		font-size: calc(var(--avatar-size) * 0.48);
		font-weight: 600;
		line-height: 1;
		user-select: none;
		flex-shrink: 0;
		transition:
			background-color 0.2s ease,
			color 0.2s ease;

		&.placeholder {
			background: var(--clr-background-2);
			color: transparent;
		}
	}
</style>
