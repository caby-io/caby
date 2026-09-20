<script lang="ts">
	import { onMount } from 'svelte';
	import IconLucideMonitor from '~icons/lucide/monitor';
	import IconLucideSunMedium from '~icons/lucide/sun-medium';
	import IconLucideMoon from '~icons/lucide/moon';
	import { colorScheme, setMode, initColorScheme, type ThemeMode } from '$lib/color-scheme.svelte';

	const MODES = [
		{ value: 'auto', label: 'System', Icon: IconLucideMonitor },
		{ value: 'light', label: 'Light', Icon: IconLucideSunMedium },
		{ value: 'dark', label: 'Dark', Icon: IconLucideMoon }
	] as const satisfies { value: ThemeMode; label: string; Icon: unknown }[];

	onMount(initColorScheme);
</script>

<div class="theme-select fx fx--ac" role="radiogroup" aria-label="Select theme">
	{#each MODES as { value, label, Icon } (value)}
		<label class="option fx fx--cc" class:selected={colorScheme.mode === value}>
			<input
				type="radio"
				name="theme"
				{value}
				checked={colorScheme.mode === value}
				onchange={() => setMode(value)}
			/>
			<Icon />
			<span class="sr-only">{label}</span>
		</label>
	{/each}
</div>

<style lang="scss">
	.theme-select {
		gap: 0.1rem;
		padding: 0.15rem;
		border-radius: 999px;
		border: 1px solid var(--clr-border);
		background-color: var(--clr-background-2);
	}

	.option {
		position: relative;
		width: 1.6rem;
		height: 1.6rem;
		border-radius: 999px;
		color: var(--clr-text-2);
		cursor: pointer;
		transition:
			background-color 0.15s ease,
			color 0.15s ease;

		:global(svg) {
			width: 0.8rem;
			height: 0.8rem;
		}

		&:hover {
			color: var(--clr-text-0);
		}

		&.selected {
			background-color: color-mix(in oklab, var(--clr-accent) 25%, transparent);
			color: var(--clr-accent);
		}

		&:has(input:focus-visible) {
			outline: 2px solid var(--clr-accent);
			outline-offset: 2px;
		}

		input {
			all: unset;
			position: absolute;
			width: 1px;
			height: 1px;
			overflow: hidden;
			clip: rect(0, 0, 0, 0);
			white-space: nowrap;
		}
	}

	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
	}
</style>
