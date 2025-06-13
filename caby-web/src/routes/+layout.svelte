<script lang="ts">
	import { toggleScheme, clearStorage } from '$lib/color-scheme';
</script>

<svelte:head>
	<script>
		let scheme = localStorage.color_scheme;
		if (!scheme) {
			scheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
		}
		document.documentElement.setAttribute('data-theme', scheme);
	</script>
</svelte:head>

<div class="sidebar fx fx--col">
	<header class="fx">
		<h1>CABY</h1>
		<!-- todo: toggle button -->
	</header>
	<nav>
		<button on:click={toggleScheme}>Toggle Scheme</button>
		<button on:click={clearStorage}>Clear Storage</button>
	</nav>
	<nav class="sidebar__nav fx-grow fx fx--col">
		<a href="/"><h2>Home</h2></a>
		<a href="/files"><h2>Files</h2></a>
		<a class="indent-1"><h2>Uploads</h2></a>
	</nav>
	<div class="usage-metrics">
		<header class="fx fx-cc">
			<h1 class="fx-grow">Disk Usage (Fake)</h1>
			<span>1GB/20GB (5%)</span>
		</header>
		<div><span></span></div>
	</div>
</div>

<slot />

<style lang="scss">
	@import '../styles.css';

	:global(body) {
		// Desktop
		display: flex;
		flex-direction: row;
	}

	.sidebar {
		width: 20rem;

		// Desktop styles
		border-right: 1px solid var(--clr-accent);
		min-height: 100vh;

		.sidebar__nav {
			> a {
				margin-inline: 1rem;
				padding: 0.5rem 1rem;
				border-radius: 5px;
				text-decoration: none;
				color: var(--clr-text);

				&.indent-1 {
					margin-left: 2rem;
				}

				// background: var(--clr-secondary);
				> h2 {
					font-size: 1.1rem;
					font-family: 'Mako', sans-serif;
					font-weight: bold;
				}

				&:hover {
					color: color-mix(in srgb, var(--clr-text), transparent 30%);
				}
			}
		}
	}

	header {
		// height: var(--top-nav-height);
		// border-bottom: 1px solid var(--clr-accent);
		padding: 0 1rem;
		box-sizing: border-box; // Guarantee that the border doesn't contribute to the height
		align-items: center;

		// Desktop styles

		h1 {
			color: var(--clr-title);
			// flex-grow: 1;
			font-family: 'Oswald', sans-serif;
			font-optical-sizing: auto;
			font-weight: 700;
			font-style: normal;
		}

		button {
			margin-right: 1rem;

			&:last-of-type {
				margin-right: 0;
			}
		}
	}

	.usage-metrics {
		padding: 2rem 1rem;

		> header {
			> h1 {
				font-size: 0.9rem; // todo: replace with var
				font-weight: normal;
			}

			> span {
				font-size: 0.6rem;
			}
		}

		> div {
			width: 100%;
			height: 0.2rem;
			border-radius: 0.1rem;
			display: flex;
			background: var(--clr-text);

			> span {
				// temp
				width: 5%;
				height: 100%;
				background-color: var(--clr-secondary);
			}
		}
	}
</style>
