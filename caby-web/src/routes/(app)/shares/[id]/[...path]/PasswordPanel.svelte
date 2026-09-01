<script lang="ts">
	let {
		onSubmit,
		error,
		loading = false
	}: {
		onSubmit: (password: string) => void;
		error?: string;
		loading?: boolean;
	} = $props();

	let password = $state('');
</script>

<div class="password-panel fx fx--col box-shadow-0-card border-0">
	<header class="title fx fx--ac">
		<h3 class="fx-grow">Password required</h3>
	</header>
	<form
		class="fx fx--col"
		onsubmit={(e) => {
			e.preventDefault();
			onSubmit(password);
		}}
	>
		<label for="share-password">This share is protected. Enter its password to continue.</label>
		<input id="share-password" class:error type="password" bind:value={password} />
		<span class="error-message">{error ?? ''}</span>
		<div class="actions fx">
			<button class="button primary fx-grow" disabled={loading} type="submit">Unlock</button>
		</div>
	</form>
</div>

<style lang="scss">
	.password-panel {
		margin: auto;
		background-color: var(--clr-background-1);
		width: fit-content;
		min-width: 20rem;

		header.title {
			background: var(--clr-background-2);
			padding: 1rem;
			border-bottom: 1px solid var(--clr-border);
		}

		> form {
			margin: 0.25rem 1rem 1rem;

			> label {
				margin: 0.25em 0;
			}

			.actions {
				margin-top: 1.5rem;
				text-align: center;
			}

			.error-message {
				font-size: 0.7rem;
				color: var(--clr-error);

				&::first-letter {
					text-transform: uppercase;
				}
			}
		}
	}
</style>
