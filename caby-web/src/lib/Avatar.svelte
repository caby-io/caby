<script lang="ts">
	let { name, size = '2rem' }: { name: string; size?: string } = $props();

	const initial = $derived((name.trim()[0] ?? '?').toUpperCase());
	const hue = $derived(
		(([...name].reduce((h, c) => (c.charCodeAt(0) + ((h << 5) - h)) | 0, 0) % 360) + 360) % 360
	);
</script>

<span class="avatar fx fx--cc" style="--avatar-size:{size}; --avatar-hue:{hue}" aria-hidden="true">
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
	}
</style>
