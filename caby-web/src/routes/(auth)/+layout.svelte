<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<div class="auth-view">
	{@render children()}
</div>

<style lang="scss">
	.auth-view {
		min-height: 100vh;
		align-content: center;
	}
</style>
