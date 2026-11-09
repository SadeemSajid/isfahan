<script lang="ts">
	import { onMount } from 'svelte';
	import WorldCanvas from '$lib/components/WorldCanvas.svelte';
	import SpeechBubble from '$lib/components/SpeechBubble.svelte';
	import HudControls from '$lib/components/HudControls.svelte';
	import { companion } from '$lib/stores/companion.svelte';

	function isTauriRuntime(): boolean {
		return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
	}

	async function toggleClickThrough(): Promise<void> {
		const next = !companion.clickThroughEnabled;
		companion.clickThroughEnabled = next;

		if (isTauriRuntime()) {
			const { invoke } = await import('@tauri-apps/api/core');
			await invoke('set_click_through', { ignore: next });
		}
	}

	async function toggleAutostart(): Promise<void> {
		if (!isTauriRuntime()) {
			companion.autostartEnabled = !companion.autostartEnabled;
			return;
		}
		try {
			const { enable, disable, isEnabled } = await import('@tauri-apps/plugin-autostart');
			if (await isEnabled()) {
				await disable();
				companion.autostartEnabled = false;
			} else {
				await enable();
				companion.autostartEnabled = true;
			}
		} catch {
			// Ignore if autostart is restricted in dev mode
		}
	}

	onMount(() => {
		let unlistenTauri: (() => void) | null = null;
		let decayTimer: ReturnType<typeof setInterval> | null = null;

		companion.loadData().then(() => {
			// Show an initial welcoming quote after data loads
			companion.showRandomQuote();
		});

		// Check for midnight day rollover once per minute
		decayTimer = setInterval(() => {
			companion.decayStep();
		}, 60_000);

		if (isTauriRuntime()) {
			(async () => {
				const { listen } = await import('@tauri-apps/api/event');
				unlistenTauri = await listen('keystroke_tick', () => {
					companion.registerKeystrokeTick();
				});

				try {
					const { isEnabled } = await import('@tauri-apps/plugin-autostart');
					companion.autostartEnabled = await isEnabled();
				} catch {
					// Autostart check optional during dev
				}
			})();
		} else {
			// Browser dev fallback: listen for keydown without reading key payload
			const onBrowserKeydown = () => {
				companion.registerKeystrokeTick();
			};
			window.addEventListener('keydown', onBrowserKeydown);
			unlistenTauri = () => window.removeEventListener('keydown', onBrowserKeydown);
		}

		return () => {
			if (unlistenTauri) unlistenTauri();
			if (decayTimer) clearInterval(decayTimer);
		};
	});
</script>

<main class="overlay-stage" data-tauri-drag-region>
	<SpeechBubble />
	<WorldCanvas />
	<HudControls onToggleClickThrough={toggleClickThrough} onToggleAutostart={toggleAutostart} />
</main>

<style>
	.overlay-stage {
		position: relative;
		width: 384px;
		height: 288px;
		overflow: hidden;
		background: transparent;
	}
</style>
