<script lang="ts">
	import AuthCard from '$lib/AuthCard.svelte';

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

<AuthCard title="Password required">
	<form
		class="password-form fx fx--col"
		onsubmit={(e) => {
			e.preventDefault();
			onSubmit(password);
		}}
	>
		<p id="share-password-hint" class="hint">
			This share is protected. Enter its password to continue.
		</p>
		<label for="share-password">Password</label>
		<input
			id="share-password"
			class:error
			type="password"
			aria-describedby="share-password-hint"
			bind:value={password}
		/>
		<span class="error-message">{error ?? ''}</span>
		<div class="actions fx">
			<button class="button primary fx-grow" disabled={loading} type="submit">Unlock</button>
		</div>
	</form>
</AuthCard>

<style lang="scss">
	.password-form {
		margin: 0.75rem 1rem 1rem;

		> .hint {
			margin: 0 0 1rem;
			font-size: 0.9rem;
			color: var(--clr-text-2);
		}

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
</style>
