<script lang="ts">
	import { companion, CRITTER_OPTIONS } from '$lib/stores/companion.svelte';

	interface Props {
		onToggleClickThrough: () => void;
		onToggleAutostart: () => void;
	}

	let { onToggleClickThrough, onToggleAutostart }: Props = $props();
</script>

<div class="stage-label-above">
	<span class="stage-name">{companion.currentStage.name}</span>
</div>

<div class="hud-bar" data-tauri-drag-region>
	<div class="stage-info" data-tauri-drag-region>
		<div
			class="meter-track"
			title="{companion.wordsToday} / 1000 words today ({Math.round(companion.happiness)}%) — {companion.nextStage ? `${companion.wordsToNextStage} words to ${companion.nextStage.name}` : 'Max stage reached!'}"
		>
			<div class="meter-fill" style="width: {companion.happiness}%"></div>
		</div>
	</div>

	<div class="critter-picker" role="group" aria-label="Choose companion critter">
		{#each CRITTER_OPTIONS as opt (opt.id)}
			<button
				type="button"
				class="critter-btn"
				class:selected={companion.selectedCritter === opt.id}
				onclick={() => companion.selectCritter(opt.id)}
				title="Companion: {opt.title}"
			>
				{opt.name}
			</button>
		{/each}
	</div>

	<div class="hud-actions">
		<button
			type="button"
			class="hud-btn"
			onclick={() => companion.showRandomQuote()}
			title="Inspire me with a quote"
		>
			Wisdom
		</button>
		<button
			type="button"
			class="hud-btn"
			class:active={companion.autostartEnabled}
			onclick={onToggleAutostart}
			title="Toggle Run on Login"
		>
			Auto
		</button>
		<button
			type="button"
			class="hud-btn"
			class:active={companion.clickThroughEnabled}
			onclick={onToggleClickThrough}
			title="Pass mouse clicks through overlay"
		>
			Ghost
		</button>
	</div>
</div>

<style>
	.hud-bar {
		position: absolute;
		bottom: 4px;
		left: 6px;
		right: 6px;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 5px;
		background: rgba(24, 20, 17, 0.9);
		border: 1px solid #8c7a6b;
		padding: 3px 5px;
		font-size: 10px;
		color: #e8dfd1;
		opacity: 0.28;
		transition: opacity 0.18s ease;
		cursor: grab;
		pointer-events: auto;
		z-index: 30;
	}

	.hud-bar:hover {
		opacity: 1;
	}

	.stage-label-above {
		position: absolute;
		bottom: 30px;
		left: 8px;
		pointer-events: none;
		z-index: 25;
	}

	.stage-name {
		display: inline-block;
		background: rgba(24, 20, 17, 0.78);
		border: 1px solid #8c7a6b;
		padding: 1px 6px;
		font-size: 10px;
		font-weight: bold;
		color: #f5c842;
		white-space: nowrap;
		text-shadow: 1px 1px 0 #14100d;
	}

	.stage-info {
		display: flex;
		align-items: center;
		gap: 5px;
		min-width: 0;
	}

	.meter-track {
		width: 44px;
		height: 6px;
		background: #362e28;
		border: 1px solid #6b5b4d;
		flex-shrink: 0;
	}

	.meter-fill {
		height: 100%;
		background: #4bb543;
		transition: width 0.2s ease;
	}

	.critter-picker {
		display: flex;
		gap: 3px;
	}

	.critter-btn {
		background: #231d19;
		color: #b9afa3;
		border: 1px solid #574a40;
		padding: 1px 5px;
		font-family: inherit;
		font-size: 9px;
		cursor: pointer;
	}

	.critter-btn:hover {
		border-color: #d4af37;
		color: #fff9ea;
	}

	.critter-btn.selected {
		background: #4e6b3a;
		border-color: #9de06c;
		color: #ffffff;
		font-weight: bold;
	}

	.hud-actions {
		display: flex;
		gap: 3px;
	}

	.hud-btn {
		background: #2b241f;
		color: #d6cec2;
		border: 1px solid #6b5b4d;
		padding: 1px 5px;
		font-family: inherit;
		font-size: 9px;
		cursor: pointer;
	}

	.hud-btn:hover {
		border-color: #f5c842;
		color: #ffffff;
	}

	.hud-btn.active {
		background: #1b8a99;
		border-color: #a8f5ff;
		color: #ffffff;
	}
</style>
