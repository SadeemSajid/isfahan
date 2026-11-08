import type { CritterId, CritterOption, Quote, SpriteManifest, WorldStage } from '$lib/types/world';

const CRITTER_STORAGE_KEY = 'isfahan.selectedCritter';
const DAILY_PROGRESS_STORAGE_KEY = 'isfahan.dailyProgress';

export const KEYSTROKES_PER_WORD = 5;
export const DAILY_WORD_GOAL = 1000;
export const DAILY_KEYSTROKE_GOAL = DAILY_WORD_GOAL * KEYSTROKES_PER_WORD; // 5,000 keystrokes = 1,000 words = 100%
export const QUOTE_KEYSTROKE_INTERVAL = 150; // +150 keystrokes (~30 words) per quote

function getTodayDateKey(): string {
	const now = new Date();
	const yyyy = now.getFullYear();
	const mm = String(now.getMonth() + 1).padStart(2, '0');
	const dd = String(now.getDate()).padStart(2, '0');
	return `${yyyy}-${mm}-${dd}`;
}

export const CRITTER_OPTIONS: CritterOption[] = [
	{ id: 'stag', name: 'Stag', title: 'Maral Stag' },
	{ id: 'badger', name: 'Badger', title: 'Forager Badger' },
	{ id: 'wolf', name: 'Wolf', title: 'Steppe Wolf' }
];

const DEFAULT_STAGES: WorldStage[] = [
	{
		id: 'barren',
		name: 'Quiet Earth',
		minHappiness: 0,
		companionMood: 'sleepy',
		palette: {
			ground: '#6b5b4d',
			accent: '#8c7a6b',
			foliage: '#5e7153',
			water: '#4a6b82'
		},
		unlockedProps: ['stone_lantern']
	}
];

class CompanionEngine {
	happiness = $state(0);
	totalKeystrokes = $state(0);
	lastKeystrokeAt = $state(0);
	isTypingBurst = $state(false);
	activeQuote = $state<Quote | null>(null);
	clickThroughEnabled = $state(false);
	autostartEnabled = $state(false);
	selectedCritter = $state<CritterId>('stag');

	quotes = $state<Quote[]>([]);
	stages = $state<WorldStage[]>(DEFAULT_STAGES);
	manifest = $state<SpriteManifest | null>(null);

	private currentDayKey = getTodayDateKey();
	private nextQuoteMilestone = QUOTE_KEYSTROKE_INTERVAL;
	private quoteDismissTimer: ReturnType<typeof setTimeout> | null = null;
	private burstResetTimer: ReturnType<typeof setTimeout> | null = null;
	private persistTimer: ReturnType<typeof setTimeout> | null = null;

	wordsToday = $derived(Math.floor(this.totalKeystrokes / KEYSTROKES_PER_WORD));

	currentStage = $derived.by(() => {
		let matched = this.stages[0] ?? DEFAULT_STAGES[0];
		for (const stage of this.stages) {
			if (this.happiness >= stage.minHappiness) {
				matched = stage;
			}
		}
		return matched;
	});

	nextStage = $derived.by(() => {
		for (const stage of this.stages) {
			if (stage.minHappiness > this.happiness) {
				return stage;
			}
		}
		return null;
	});

	wordsToNextStage = $derived.by(() => {
		if (!this.nextStage) return 0;
		const targetWords = Math.round((this.nextStage.minHappiness / 100) * DAILY_WORD_GOAL);
		return Math.max(0, targetWords - this.wordsToday);
	});

	async loadData(): Promise<void> {
		if (typeof window !== 'undefined') {
			const saved = window.localStorage.getItem(CRITTER_STORAGE_KEY) as CritterId | null;
			if (saved && CRITTER_OPTIONS.some((c) => c.id === saved)) {
				this.selectedCritter = saved;
			}

			try {
				const rawProgress = window.localStorage.getItem(DAILY_PROGRESS_STORAGE_KEY);
				if (rawProgress) {
					const parsed = JSON.parse(rawProgress) as { date?: string; keystrokes?: number };
					const today = getTodayDateKey();
					if (parsed.date === today && typeof parsed.keystrokes === 'number' && parsed.keystrokes >= 0) {
						this.currentDayKey = today;
						this.totalKeystrokes = parsed.keystrokes;
						this.happiness = Math.min(
							100,
							+((this.totalKeystrokes / DAILY_KEYSTROKE_GOAL) * 100).toFixed(2)
						);
						this.nextQuoteMilestone =
							(Math.floor(this.totalKeystrokes / QUOTE_KEYSTROKE_INTERVAL) + 1) *
							QUOTE_KEYSTROKE_INTERVAL;
					}
				}
			} catch {
				// Ignore malformed local storage
			}
		}

		try {
			const [quotesRes, stagesRes, manifestRes] = await Promise.all([
				fetch('/data/quotes.json'),
				fetch('/data/world-stages.json'),
				fetch('/data/sprite-manifest.json')
			]);
			if (quotesRes.ok) {
				this.quotes = (await quotesRes.json()) as Quote[];
			}
			if (stagesRes.ok) {
				const loadedStages = (await stagesRes.json()) as WorldStage[];
				if (loadedStages.length > 0) {
					this.stages = loadedStages;
				}
			}
			if (manifestRes.ok) {
				this.manifest = (await manifestRes.json()) as SpriteManifest;
			}
		} catch {
			// Offline static assets load locally; keep defaults if fetch fails
		}
	}

	selectCritter(critter: CritterId): void {
		this.selectedCritter = critter;
		if (typeof window !== 'undefined') {
			window.localStorage.setItem(CRITTER_STORAGE_KEY, critter);
		}
	}

	private scheduleSaveDailyProgress(): void {
		if (typeof window === 'undefined' || this.persistTimer) return;
		this.persistTimer = setTimeout(() => {
			this.persistTimer = null;
			try {
				window.localStorage.setItem(
					DAILY_PROGRESS_STORAGE_KEY,
					JSON.stringify({
						date: this.currentDayKey,
						keystrokes: this.totalKeystrokes
					})
				);
			} catch {
				// Ignore storage quota errors
			}
		}, 1200);
	}

	private checkDayRollover(): void {
		const today = getTodayDateKey();
		if (today !== this.currentDayKey) {
			this.currentDayKey = today;
			this.totalKeystrokes = 0;
			this.happiness = 0;
			this.nextQuoteMilestone = QUOTE_KEYSTROKE_INTERVAL;
			this.scheduleSaveDailyProgress();
		}
	}

	/**
	 * Called whenever a privacy-safe keystroke tick occurs.
	 * Intentionally accepts no arguments so key data can never enter state.
	 */
	registerKeystrokeTick(): void {
		this.checkDayRollover();

		this.totalKeystrokes += 1;
		this.lastKeystrokeAt = Date.now();
		this.isTypingBurst = true;

		// 5,000 keystrokes (1,000 words at 5 keys/word) = 100% daily progression
		this.happiness = Math.min(
			100,
			+((this.totalKeystrokes / DAILY_KEYSTROKE_GOAL) * 100).toFixed(2)
		);

		this.scheduleSaveDailyProgress();

		if (this.burstResetTimer) {
			clearTimeout(this.burstResetTimer);
		}
		this.burstResetTimer = setTimeout(() => {
			this.isTypingBurst = false;
		}, 900);

		// Trigger wisdom quote every +150 keystrokes (~30 words)
		if (this.totalKeystrokes >= this.nextQuoteMilestone) {
			this.nextQuoteMilestone += QUOTE_KEYSTROKE_INTERVAL;
			this.showRandomQuote();
		}
	}

	/**
	 * Checks for midnight rollover so each new day starts a fresh 1,000-word sanctuary journey.
	 */
	decayStep(): void {
		this.checkDayRollover();
	}

	showRandomQuote(): void {
		const eligible = this.quotes.filter((q) => q.minHappiness <= this.happiness);
		const pool = eligible.length > 0 ? eligible : this.quotes;
		if (pool.length === 0) return;

		const pick = pool[Math.floor(Math.random() * pool.length)];
		this.activeQuote = pick;

		if (this.quoteDismissTimer) {
			clearTimeout(this.quoteDismissTimer);
		}
		this.quoteDismissTimer = setTimeout(() => {
			this.activeQuote = null;
		}, 11_000);
	}

	dismissQuote(): void {
		if (this.quoteDismissTimer) {
			clearTimeout(this.quoteDismissTimer);
		}
		this.activeQuote = null;
	}
}

export const companion = new CompanionEngine();
