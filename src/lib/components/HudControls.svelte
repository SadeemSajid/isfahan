<script lang="ts">
	import { companion } from '$lib/stores/companion.svelte';

	interface Props {
		onToggleClickThrough: () => void;
		onToggleAutostart: () => void;
	}

	let { onToggleClickThrough, onToggleAutostart }: Props = $props();
</script>

<div class="stage-label-above" class:collapsed={companion.hudCollapsed}>
	<span class="stage-name">{companion.currentStage.name}</span>
</div>

{#if companion.hudCollapsed}
	<button
		type="button"
		class="pixel-collapse-btn expand-floating"
		onclick={() => companion.toggleHudCollapsed()}
		title="Expand bottom bar"
		aria-label="Expand bottom bar"
	>
		<svg width="7" height="4" viewBox="0 0 7 4" shape-rendering="crispEdges" aria-hidden="true">
			<rect x="3" y="0" width="1" height="1" fill="currentColor" />
			<rect x="2" y="1" width="1" height="1" fill="currentColor" />
			<rect x="4" y="1" width="1" height="1" fill="currentColor" />
			<rect x="1" y="2" width="1" height="1" fill="currentColor" />
			<rect x="5" y="2" width="1" height="1" fill="currentColor" />
		</svg>
	</button>
{:else}
	<div class="hud-bar" data-tauri-drag-region>
		<div class="stage-info" data-tauri-drag-region>
			<button
				type="button"
				class="meter-track"
				onclick={(e) => {
					if (e.shiftKey) {
						companion.addTestWords(200);
					}
				}}
				title="{companion.wordsToday} / 1000 words today ({Math.round(companion.happiness)}%) — {companion.nextStage ? `${companion.wordsToNextStage} words to ${companion.nextStage.name}` : 'Max stage reached!'} (Shift+Click to add +200 test words)"
			>
				<div class="meter-fill" style="width: {companion.happiness}%"></div>
			</button>
			<button
				type="button"
				class="pixel-reset-btn"
				onclick={() => companion.resetDailyProgress()}
				title="Reset daily word progress to 0 (Shift+Click bar to add +200 test words)"
				aria-label="Reset daily word progress"
			>
				↺
			</button>
		</div>

		<div class="cycle-controls">
			<button
				type="button"
				class="map-btn selected"
				onclick={() => companion.cycleMap()}
				title="Map: {companion.currentMapOption.title} (click to switch)"
			>
				{companion.currentMapOption.name}
			</button>

			<button
				type="button"
				class="critter-btn selected"
				onclick={() => companion.cycleCritter()}
				title="Companion: {companion.currentCritterOption.title} (click to switch)"
			>
				{companion.currentCritterOption.name}
			</button>
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
			<button
				type="button"
				class="pixel-collapse-btn"
				onclick={() => companion.toggleHudCollapsed()}
				title="Collapse bottom bar"
				aria-label="Collapse bottom bar"
			>
				<svg width="7" height="4" viewBox="0 0 7 4" shape-rendering="crispEdges" aria-hidden="true">
					<rect x="1" y="1" width="1" height="1" fill="currentColor" />
					<rect x="5" y="1" width="1" height="1" fill="currentColor" />
					<rect x="2" y="2" width="1" height="1" fill="currentColor" />
					<rect x="4" y="2" width="1" height="1" fill="currentColor" />
					<rect x="3" y="3" width="1" height="1" fill="currentColor" />
				</svg>
			</button>
		</div>
	</div>
{/if}

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
		transition:
			bottom 0.18s ease,
			opacity 0.18s ease;
	}

	.stage-label-above.collapsed {
		bottom: 5px;
		opacity: 0.55;
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
		gap: 4px;
		min-width: 0;
	}

	.meter-track {
		width: 30px;
		height: 6px;
		padding: 0;
		background: #362e28;
		border: 1px solid #6b5b4d;
		flex-shrink: 0;
		cursor: pointer;
		display: block;
	}

	.meter-track:hover {
		border-color: #f5c842;
	}

	.pixel-reset-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 13px;
		height: 13px;
		padding: 0;
		background: #231d19;
		color: #b9afa3;
		border: 1px solid #574a40;
		font-family: inherit;
		font-size: 9px;
		line-height: 1;
		cursor: pointer;
	}

	.pixel-reset-btn:hover {
		border-color: #f5c842;
		color: #f5c842;
	}

	.meter-fill {
		height: 100%;
		background: #4bb543;
		transition: width 0.2s ease;
	}

	.cycle-controls {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.map-btn,
	.critter-btn {
		background: #231d19;
		color: #b9afa3;
		border: 1px solid #574a40;
		padding: 1px 6px;
		font-family: inherit;
		font-size: 9px;
		cursor: pointer;
	}

	.map-btn:hover,
	.critter-btn:hover {
		border-color: #d4af37;
		color: #fff9ea;
	}

	.map-btn.selected {
		background: #35586c;
		border-color: #82d8f7;
		color: #ffffff;
		font-weight: bold;
	}

	.critter-btn.selected {
		background: #4e6b3a;
		border-color: #9de06c;
		color: #ffffff;
		font-weight: bold;
	}

	.hud-actions {
		display: flex;
		align-items: center;
		gap: 2px;
	}

	.hud-btn {
		background: #2b241f;
		color: #d6cec2;
		border: 1px solid #6b5b4d;
		padding: 1px 4px;
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

	.pixel-collapse-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 15px;
		height: 14px;
		padding: 0;
		background: #231d19;
		color: #d6cec2;
		border: 1px solid #8c7a6b;
		box-shadow: inset 1px 1px 0 #3d332c;
		cursor: pointer;
	}

	.pixel-collapse-btn:hover {
		background: #3b312a;
		border-color: #f5c842;
		color: #f5c842;
	}

	.expand-floating {
		position: absolute;
		bottom: 5px;
		right: 6px;
		opacity: 0.42;
		transition: opacity 0.16s ease;
		pointer-events: auto;
		z-index: 30;
	}

	.expand-floating:hover {
		opacity: 1;
	}
</style>
