<script lang="ts">
	import 'iconify-icon';
	import { uploadManager } from '$lib/files/upload_manager.svelte';
	import { prettyBytes } from '$lib/fs';

	let progress_percent = $derived(
		Math.floor((uploadManager.upload_progress.progress * 100) / uploadManager.upload_progress.total)
	);
	let completed_bytes = $derived(prettyBytes(uploadManager.upload_progress.progress));
	let total_bytes = $derived(prettyBytes(uploadManager.upload_progress.total));

	// todo: move

	$effect(() => {
		uploadManager.upload_progress;
	});
</script>

<div class="upload-bar border-0 box-shadow-0-card">
	<header class="fx fx--ac">
		<h1>Upload Progress</h1>
		<h2>{progress_percent}%</h2>
		<span class="fx fx--cc border-0 box-shadow-0-card">
			<iconify-icon icon="lucide:chevron-up"></iconify-icon>
		</span>
	</header>
	<main class="fx fx--col">
		<section class="fx">
			<span>{uploadManager.upload_files.length} files</span>
			<span>?? Mb/s</span>
			<span>{completed_bytes} of {total_bytes}</span>
			<span>??m remaining</span>
		</section>
		<progress class="border-0 box-shadow-0-card" max="100" value={progress_percent || 0}>
			{progress_percent}%
		</progress>
	</main>
</div>

<style lang="scss">
	.upload-bar {
		background: var(--clr-background-1);
		border-radius: 3px;
		width: clamp(20rem, 40%, 40rem); // todo: clamp
		margin: 1rem 0;
		padding: 1rem;

		> header {
			h1 {
				font-size: 1rem;
				flex-grow: 1;
			}

			h2 {
				font-size: 1rem;
				font-weight: normal;
			}

			span {
				cursor: pointer;
				font-size: 1.2rem;
				margin-left: 1rem;
				width: 1.5rem;
				height: 1.5rem;
				background-color: var(--clr-background-2);
				border-radius: 50%;
			}
		}

		> main {
			font-size: 0.9em;
			margin-top: 1rem;

			> section {
				gap: 0.5rem;

				span {
					color: var(--clr-text-2);
				}

				span:first-of-type {
					color: var(--clr-text-1);
					font-weight: bold;

					&::before {
						content: none;
					}
				}

				span::before {
					margin-right: 0.5rem;
					content: '/';
				}
			}

			progress {
				margin-top: 0.5rem;
				width: 100%;
				height: 0.3rem;
				border-radius: 3px;
				overflow: hidden;

				// &[value] {
				// 	// border-radius: 3px;
				// 	// width: 250px;
				// 	color: limegreen;
				// }

				&::-webkit-progress-bar {
					background-color: var(--clr-background-2);
				}
				&::-webkit-progress-value {
					transition: width 0.3s ease;
					background-color: var(--clr-success);
				}
				&::-moz-progress-bar {
					background-color: var(--clr-success);
					transition: width 0.3s ease;
				}
			}
		}
	}
</style>
