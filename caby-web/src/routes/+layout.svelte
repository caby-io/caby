<script lang="ts">
	import type { Snippet } from 'svelte';
	// global styles
	import '../styles.css';

	let { children }: { children: Snippet } = $props();
</script>

{@render children()}
