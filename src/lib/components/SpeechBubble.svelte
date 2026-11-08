<script lang="ts">
	import { companion } from '$lib/stores/companion.svelte';
</script>

{#if companion.activeQuote}
	<div class="speech-bubble" role="status" aria-live="polite">
		<button
			type="button"
			class="close-btn"
			onclick={() => companion.dismissQuote()}
			title="Dismiss quote"
		>
			×
		</button>
		<p class="quote-text">“{companion.activeQuote.text}”</p>
		<p class="quote-meta">
			— {companion.activeQuote.author}, <span>{companion.activeQuote.source}</span>
		</p>
		<div class="bubble-tail"></div>
	</div>
{/if}

<style>
	.speech-bubble {
		position: absolute;
		top: 8px;
		left: 14px;
		right: 14px;
		background: #1c1815;
		border: 2px solid #f5c842;
		box-shadow:
			0 0 0 2px #1c1815,
			0 4px 0 rgba(0, 0, 0, 0.45);
		padding: 8px 22px 8px 10px;
		color: #f9f3e3;
		font-size: 11px;
		line-height: 1.35;
		pointer-events: auto;
		z-index: 20;
	}

	.quote-text {
		margin-bottom: 4px;
		color: #fff9ea;
	}

	.quote-meta {
		font-size: 9.5px;
		color: #f5c842;
	}

	.quote-meta span {
		color: #bfb5a2;
		font-style: italic;
	}

	.close-btn {
		position: absolute;
		top: 3px;
		right: 5px;
		background: transparent;
		border: none;
		color: #bfb5a2;
		font-size: 13px;
		cursor: pointer;
		line-height: 1;
		padding: 2px;
	}

	.close-btn:hover {
		color: #ffffff;
	}

	.bubble-tail {
		position: absolute;
		bottom: -7px;
		left: 50%;
		transform: translateX(-50%);
		width: 0;
		height: 0;
		border-left: 6px solid transparent;
		border-right: 6px solid transparent;
		border-top: 6px solid #f5c842;
	}
</style>
