<script lang="ts">
	import { onMount } from 'svelte';
	import { companion } from '$lib/stores/companion.svelte';
	import type { CritterId, IsoDirection, TileCoord } from '$lib/types/world';

	// Logical canvas dimensions (240x180 displayed at 384x288 for a crisp, slightly larger view)
	const LOGICAL_W = 240;
	const LOGICAL_H = 180;
	const TILE_SIZE = 32;
	const GRID_SIZE = 7;

	// Isometric projection origin for 7x7 grid of 32x32 tiles inside 240x180 logical canvas
	const ORIGIN_X = 104;
	const ORIGIN_Y = 38;

	let canvasEl: HTMLCanvasElement | undefined = $state();
	let tilesetImg: HTMLImageElement | null = null;

	type CritterAction = 'idle' | 'walk' | 'run';
	type CritterSheetMap = Record<
		CritterId,
		Record<CritterAction, Record<IsoDirection, HTMLImageElement | null>>
	>;

	const dirs: IsoDirection[] = ['SE', 'SW', 'NE', 'NW'];

	const critterImages: CritterSheetMap = {
		stag: {
			idle: { SE: null, SW: null, NE: null, NW: null },
			walk: { SE: null, SW: null, NE: null, NW: null },
			run: { SE: null, SW: null, NE: null, NW: null }
		},
		badger: {
			idle: { SE: null, SW: null, NE: null, NW: null },
			walk: { SE: null, SW: null, NE: null, NW: null },
			run: { SE: null, SW: null, NE: null, NW: null }
		},
		wolf: {
			idle: { SE: null, SW: null, NE: null, NW: null },
			walk: { SE: null, SW: null, NE: null, NW: null },
			run: { SE: null, SW: null, NE: null, NW: null }
		}
	};

	// Walkable meadow ring around the central 3x3 Oasis Pond (gx = 2..4, gy = 2..4)
	const OASIS_WALKABLE_CELLS: [number, number][] = [
		[1, 2],
		[1, 3],
		[1, 4],
		[1, 5],
		[2, 5],
		[3, 5],
		[4, 5],
		[5, 5],
		[5, 4],
		[5, 3],
		[5, 2],
		[5, 1],
		[4, 1],
		[3, 1],
		[2, 1],
		[2, 6],
		[3, 6]
	];

	// Walkable cobblestone promenade on the Stone Isle (front/left stone terrace)
	const STONE_ISLE_WALKABLE_CELLS: [number, number][] = [
		[2, 2],
		[2, 3],
		[2, 4],
		[3, 4],
		[4, 4],
		[2, 5],
		[3, 5]
	];

	function isWalkable(gx: number, gy: number): boolean {
		const cells =
			companion.selectedMap === 'stone_isle' ? STONE_ISLE_WALKABLE_CELLS : OASIS_WALKABLE_CELLS;
		return cells.some(([wx, wy]) => wx === gx && wy === gy);
	}

	// Autonomous critter state on the isometric grid
	let critterGx = 2;
	let critterGy = 4;
	let targetGx = 2;
	let targetGy = 4;
	let moveProgress = 0;
	let isMoving = false;
	let critterDir: IsoDirection = 'SE';
	let idleTicksRemaining = 48;
	let animFrame = 0;
	let renderSubTick = 0;
	let lastMapId = companion.selectedMap;

	// Realistic Ambient Particle System (fireflies, drifting blossom petals, water glints, typing motes)
	interface AmbientParticle {
		kind: 'firefly' | 'petal' | 'glint' | 'mote';
		x: number;
		y: number;
		vx: number;
		vy: number;
		phase: number;
		phaseSpeed: number;
		life: number;
		maxLife: number;
		colorCore: string;
		colorGlow: string;
	}

	const particles: AmbientParticle[] = [];

	function spawnAmbientParticle(kind: AmbientParticle['kind'], originX?: number, originY?: number): AmbientParticle {
		if (kind === 'firefly') {
			return {
				kind,
				x: 42 + Math.random() * 156,
				y: 44 + Math.random() * 98,
				vx: (Math.random() - 0.5) * 0.32,
				vy: -0.09 - Math.random() * 0.2,
				phase: Math.random() * Math.PI * 2,
				phaseSpeed: 0.11 + Math.random() * 0.08,
				life: 0,
				maxLife: 85 + Math.random() * 75,
				colorCore: '#fffbe0',
				colorGlow: '#ffdc69'
			};
		}
		if (kind === 'petal') {
			const isRose = Math.random() > 0.45;
			return {
				kind,
				x: 36 + Math.random() * 160,
				y: 34 + Math.random() * 40,
				vx: 0.22 + Math.random() * 0.24,
				vy: 0.24 + Math.random() * 0.2,
				phase: Math.random() * Math.PI * 2,
				phaseSpeed: 0.14 + Math.random() * 0.07,
				life: 0,
				maxLife: 75 + Math.random() * 55,
				colorCore: isRose ? '#ff9ebb' : '#ffe180',
				colorGlow: isRose ? '#e84a5f' : '#f5c842'
			};
		}
		if (kind === 'glint') {
			if (companion.selectedMap === 'stone_isle') {
				const angle = Math.random() * Math.PI * 2;
				const rx = 50 + Math.random() * 24;
				const ry = 24 + Math.random() * 14;
				return {
					kind,
					x: 118 + Math.cos(angle) * rx,
					y: 92 + Math.sin(angle) * ry,
					vx: 0,
					vy: 0,
					phase: 0,
					phaseSpeed: 0.18,
					life: 0,
					maxLife: 28 + Math.random() * 22,
					colorCore: '#ffffff',
					colorGlow: '#a8f5ff'
				};
			}
			// Shimmering light glints on the 3x3 pond surface
			return {
				kind,
				x: 94 + (Math.random() - 0.5) * 48,
				y: 88 + (Math.random() - 0.5) * 20,
				vx: 0,
				vy: 0,
				phase: 0,
				phaseSpeed: 0.18,
				life: 0,
				maxLife: 28 + Math.random() * 22,
				colorCore: '#ffffff',
				colorGlow: '#a8f5ff'
			};
		}
		// 'mote' — rising inspiration spark around the critter when typing
		return {
			kind: 'mote',
			x: (originX ?? 120) + (Math.random() - 0.5) * 14,
			y: (originY ?? 100) - 8 + (Math.random() - 0.5) * 6,
			vx: (Math.random() - 0.5) * 0.28,
			vy: -0.48 - Math.random() * 0.3,
			phase: Math.random() * Math.PI * 2,
			phaseSpeed: 0.2,
			life: 0,
			maxLife: 22 + Math.random() * 14,
			colorCore: Math.random() > 0.4 ? '#ffe98a' : '#ff8da6',
			colorGlow: '#ffb478'
		};
	}

	function updateAndDrawParticles(ctx: CanvasRenderingContext2D): void {
		const h = companion.happiness;

		// Target counts scale organically with world happiness
		const targetFireflies = h >= 75 ? 8 : h >= 40 ? 4 : 2;
		const targetPetals = h >= 65 ? 4 : h >= 35 ? 2 : 0;
		const targetGlints = h >= 20 ? 3 : 0;

		let fireflyCount = 0;
		let petalCount = 0;
		let glintCount = 0;

		for (const p of particles) {
			if (p.kind === 'firefly') fireflyCount++;
			else if (p.kind === 'petal') petalCount++;
			else if (p.kind === 'glint') glintCount++;
		}

		if (fireflyCount < targetFireflies && Math.random() < 0.22) {
			particles.push(spawnAmbientParticle('firefly'));
		}
		if (petalCount < targetPetals && Math.random() < 0.14) {
			particles.push(spawnAmbientParticle('petal'));
		}
		if (glintCount < targetGlints && Math.random() < 0.2) {
			particles.push(spawnAmbientParticle('glint'));
		}

		// Spawn rising inspiration motes around critter during typing bursts
		if (companion.isTypingBurst && Math.random() < 0.55) {
			const curGx = isMoving ? critterGx + (targetGx - critterGx) * moveProgress : critterGx;
			const curGy = isMoving ? critterGy + (targetGy - critterGy) * moveProgress : critterGy;
			const cPos = isoToScreen(curGx, curGy, 0);
			particles.push(spawnAmbientParticle('mote', cPos.x + 16, cPos.y - 8));
		}

		// Zero-allocation particle rendering using globalAlpha + static hex colors
		for (let i = particles.length - 1; i >= 0; i--) {
			const p = particles[i];
			p.life += 1;
			if (p.life >= p.maxLife) {
				particles.splice(i, 1);
				continue;
			}

			p.phase += p.phaseSpeed;
			const envelope = Math.sin((p.life / p.maxLife) * Math.PI);

			if (p.kind === 'firefly') {
				p.x += p.vx + Math.sin(p.phase) * 0.28;
				p.y += p.vy + Math.cos(p.phase * 0.7) * 0.12;

				const pulse = 0.45 + 0.55 * (0.5 + 0.5 * Math.sin(p.phase * 1.4));
				const alpha = envelope * pulse;
				const px = Math.round(p.x);
				const py = Math.round(p.y);

				ctx.fillStyle = p.colorGlow;
				ctx.globalAlpha = alpha * 0.28;
				ctx.fillRect(px - 2, py - 1, 5, 3);
				ctx.fillRect(px - 1, py - 2, 3, 5);

				ctx.globalAlpha = alpha * 0.55;
				ctx.fillRect(px - 1, py - 1, 3, 3);

				ctx.globalAlpha = alpha;
				ctx.fillStyle = p.colorCore;
				ctx.fillRect(px, py, 1, 1);
			} else if (p.kind === 'petal') {
				p.x += p.vx + Math.sin(p.phase) * 0.35;
				p.y += p.vy;

				const px = Math.round(p.x);
				const py = Math.round(p.y);

				ctx.globalAlpha = envelope * 0.85;
				ctx.fillStyle = p.colorCore;
				ctx.fillRect(px, py, 2, 1);
				ctx.fillRect(Math.sin(p.phase) > 0 ? px + 1 : px, py + 1, 1, 1);
			} else if (p.kind === 'glint') {
				const px = Math.round(p.x);
				const py = Math.round(p.y);
				const alpha = envelope * 0.75;

				ctx.fillStyle = p.colorGlow;
				ctx.globalAlpha = alpha * 0.45;
				ctx.fillRect(px - 1, py, 3, 1);

				ctx.globalAlpha = alpha;
				ctx.fillStyle = p.colorCore;
				ctx.fillRect(px, py, 1, 1);
			} else if (p.kind === 'mote') {
				p.x += p.vx + Math.sin(p.phase) * 0.22;
				p.y += p.vy;

				const px = Math.round(p.x);
				const py = Math.round(p.y);
				const alpha = envelope * 0.9;

				ctx.fillStyle = p.colorGlow;
				ctx.globalAlpha = alpha * 0.35;
				ctx.fillRect(px - 1, py - 1, 3, 3);

				ctx.globalAlpha = alpha;
				ctx.fillStyle = p.colorCore;
				ctx.fillRect(px, py, 1, 1);
			}
		}

		ctx.globalAlpha = 1;
	}

	function loadImage(src: string): Promise<HTMLImageElement | null> {
		return new Promise((resolve) => {
			const img = new Image();
			img.onload = () => resolve(img);
			img.onerror = () => resolve(null);
			img.src = src;
		});
	}

	async function preloadAssets(): Promise<void> {
		tilesetImg = await loadImage('/assets/isometric-tileset/spritesheet.png');

		const tasks: Promise<void>[] = [];

		for (const dir of dirs) {
			tasks.push(
				// Stag (32x41 frames)
				loadImage(`/assets/isometric-tileset/critters/stag/critter_stag_${dir}_idle.png`).then(
					(img) => {
						critterImages.stag.idle[dir] = img;
					}
				),
				loadImage(`/assets/isometric-tileset/critters/stag/critter_stag_${dir}_walk.png`).then(
					(img) => {
						critterImages.stag.walk[dir] = img;
					}
				),
				loadImage(`/assets/isometric-tileset/critters/stag/critter_stag_${dir}_run.png`).then(
					(img) => {
						critterImages.stag.run[dir] = img;
					}
				),
				// Badger (42x32 frames)
				loadImage(`/assets/isometric-tileset/critters/badger/critter_badger_${dir}_idle.png`).then(
					(img) => {
						critterImages.badger.idle[dir] = img;
					}
				),
				loadImage(`/assets/isometric-tileset/critters/badger/critter_badger_${dir}_walk.png`).then(
					(img) => {
						critterImages.badger.walk[dir] = img;
						critterImages.badger.run[dir] = img;
					}
				)
			);
		}

		// Wolf (4-row directional sheets: SW, SE, NW, NE — uses wolf-idle.png, never howling loop)
		const [wolfIdle, wolfRun] = await Promise.all([
			loadImage('/assets/isometric-tileset/critters/wolf/wolf-idle.png'),
			loadImage('/assets/isometric-tileset/critters/wolf/wolf-run.png')
		]);
		for (const dir of dirs) {
			critterImages.wolf.idle[dir] = wolfIdle;
			critterImages.wolf.walk[dir] = wolfRun;
			critterImages.wolf.run[dir] = wolfRun;
		}

		await Promise.all(tasks);
	}

	function isoToScreen(gx: number, gy: number, z = 0): { x: number; y: number } {
		return {
			x: Math.round(ORIGIN_X + (gx - gy) * 16),
			y: Math.round(ORIGIN_Y + (gx + gy) * 8 - z * 8)
		};
	}

	function drawSheetTile(
		ctx: CanvasRenderingContext2D,
		col: number,
		row: number,
		screenX: number,
		screenY: number
	): void {
		if (!tilesetImg) return;
		ctx.drawImage(
			tilesetImg,
			col * TILE_SIZE,
			row * TILE_SIZE,
			TILE_SIZE,
			TILE_SIZE,
			screenX,
			screenY,
			TILE_SIZE,
			TILE_SIZE
		);
	}

	/**
	 * Central 3x3 Oasis Pond at gx = 2..4, gy = 2..4.
	 */
	function isPondCell(gx: number, gy: number): boolean {
		return gx >= 2 && gx <= 4 && gy >= 2 && gy <= 4;
	}

	/**
	 * Maps each cell of the 3x3 pond (gx = 2..4, gy = 2..4) to its exact directional
	 * shoreline tile in Row 10 (col 0..8, row 10).
	 */
	function getDirectionalPondTile(gx: number, gy: number): TileCoord {
		const touchNW = gx === 2;
		const touchNE = gy === 2;
		const touchSE = gx === 4;
		const touchSW = gy === 4;

		if (touchNW && touchNE) return { col: 5, row: 10 };
		if (touchSW && touchSE) return { col: 6, row: 10 };
		if (touchNW && touchSW) return { col: 7, row: 10 };
		if (touchNE && touchSE) return { col: 8, row: 10 };
		if (touchNE) return { col: 1, row: 10 };
		if (touchNW) return { col: 2, row: 10 };
		if (touchSE) return { col: 3, row: 10 };
		if (touchSW) return { col: 4, row: 10 };
		return { col: 0, row: 10 };
	}

	function isElevatedSanctuary(gx: number, gy: number): boolean {
		return (
			(gx === 0 && gy === 0) ||
			(gx === 1 && gy === 0) ||
			(gx === 2 && gy === 0) ||
			(gx === 0 && gy === 1) ||
			(gx === 1 && gy === 1) ||
			(gx === 0 && gy === 2)
		);
	}

	function isStonePath(gx: number, gy: number): boolean {
		return (
			(gx === 3 && gy === 5) ||
			(gx === 2 && gy === 5) ||
			(gx === 4 && gy === 5) ||
			(gx === 3 && gy === 6)
		);
	}

	function getGroundTile(gx: number, gy: number): TileCoord {
		const h = companion.happiness;
		const hash = (gx * 7 + gy * 13) % 3;

		if (isPondCell(gx, gy)) {
			return getDirectionalPondTile(gx, gy);
		}

		if (isStonePath(gx, gy)) {
			if (h < 25) return { col: 3, row: 2 };
			if (h < 55) return { col: 4, row: 2 };
			return { col: 6 + ((gx + gy) % 2), row: 5 };
		}

		if (h < 20) {
			return { col: hash * 2, row: 0 };
		}

		if (h < 45) {
			if (gx >= 1 && gx <= 5 && gy >= 1 && gy <= 5) {
				return { col: 4 + (hash % 2), row: 2 };
			}
			return { col: 8 + ((gx + gy) % 2), row: 1 };
		}

		if (h < 70) {
			if ((gx + gy) % 2 === 0) {
				return { col: hash, row: 2 };
			}
			return { col: 5 + (hash % 2), row: 2 };
		}

		if ((gx + gy) % 3 === 0) {
			return { col: 4 + hash, row: 3 };
		}
		return { col: hash, row: 2 };
	}

	function getOverlayProp(gx: number, gy: number): TileCoord | null {
		const h = companion.happiness;

		if (gx === 3 && gy === 3 && h >= 30) {
			return { col: 2, row: 7 };
		}

		if (isPondCell(gx, gy)) {
			return null;
		}

		if (isWalkable(gx, gy)) {
			if (h >= 88 && ((gx === 4 && gy === 1) || (gx === 1 && gy === 4))) {
				return { col: 3, row: 4 };
			}
			return null;
		}

		if (gx === 0 && gy === 0) {
			return { col: 0, row: 6 };
		}

		if (gx === 2 && gy === 0) {
			return h >= 45 ? { col: 9, row: 5 } : { col: 10, row: 4 };
		}
		if (gx === 0 && gy === 2) {
			return h >= 45 ? { col: 10, row: 5 } : { col: 9, row: 4 };
		}

		if ((gx === 1 && gy === 0) || (gx === 0 && gy === 1)) {
			if (h >= 65) return { col: 0, row: 4 };
			if (h >= 30) return { col: 1, row: 4 };
			return null;
		}

		if (gx === 6 && gy === 1) {
			if (h >= 50) return { col: 7, row: 4 };
			if (h >= 20) return { col: 6, row: 4 };
			return { col: 4, row: 4 };
		}
		if (gx === 0 && gy === 5) {
			return h >= 45 ? { col: 8, row: 4 } : { col: 5, row: 4 };
		}

		if (gx === 6 && gy === 0) {
			return h >= 40 ? { col: 9, row: 2 } : { col: 9, row: 4 };
		}
		if (gx === 6 && gy === 2) {
			if (h >= 60) return { col: 8, row: 3 };
			if (h >= 30) return { col: 10, row: 3 };
			return null;
		}
		if (gx === 6 && gy === 3) {
			if (h >= 75) return { col: 0, row: 4 };
			if (h >= 40) return { col: 8, row: 2 };
			return null;
		}
		if (gx === 6 && gy === 4) {
			return h >= 50 ? { col: 1, row: 6 } : { col: 10, row: 5 };
		}
		if (gx === 6 && gy === 5) {
			if (h >= 70) return { col: 2, row: 4 };
			if (h >= 35) return { col: 10, row: 3 };
			return null;
		}
		if (gx === 6 && gy === 6) {
			return h >= 45 ? { col: 9, row: 5 } : { col: 9, row: 4 };
		}

		if (gx === 0 && gy === 3) {
			if (h >= 65) return { col: 8, row: 3 };
			if (h >= 25) return { col: 10, row: 3 };
			return null;
		}
		if (gx === 0 && gy === 4) {
			return h >= 50 ? { col: 10, row: 2 } : null;
		}
		if (gx === 0 && gy === 6) {
			return h >= 40 ? { col: 0, row: 4 } : { col: 1, row: 6 };
		}
		if (gx === 1 && gy === 6) {
			if (h >= 60) return { col: 2, row: 4 };
			return null;
		}
		if (gx === 4 && gy === 6) {
			if (h >= 55) return { col: 8, row: 3 };
			return null;
		}
		if (gx === 5 && gy === 6) {
			if (h >= 65) return { col: 9, row: 3 };
			return null;
		}
		if (gx === 5 && gy === 0) {
			if (h >= 70) return { col: 0, row: 4 };
			if (h >= 30) return { col: 1, row: 4 };
			return null;
		}
		if (gx === 4 && gy === 0) {
			if (h >= 60) return { col: 2, row: 4 };
			return null;
		}

		return null;
	}

	/**
	 * ============================================================================
	 * MAP 2: STONE ISLE (`stone_isle`)
	 * Inspired by the reference artwork: an organic cobblestone & lush-foliage island
	 * nestled inside a stepped dark-navy sea with animated travelling waves, glowing
	 * underwater shoals, coastal pebbles in foam rings, and twinkling water sparkles.
	 * Uses an 8x8 grid (gx = 0..7, gy = 0..7), centered in the canvas.
	 * ============================================================================
	 */
	const STONE_ISLE_GRID_SIZE = 8;
	const STONE_ISLE_OFFSET_X = 6;
	const STONE_ISLE_OFFSET_Y = -12;

	type StoneIsleCellKind = 'void' | 'water' | 'island_stone' | 'island_garden';

	function getStoneIsleCellKind(gx: number, gy: number): StoneIsleCellKind {
		// Stepped/notched outer water silhouette (like the reference image)
		if (
			(gx === 0 && gy === 0) ||
			(gx === 1 && gy === 0) ||
			(gx === 0 && gy === 1) ||
			(gx === 0 && gy === 2) ||
			(gx === 6 && gy === 0) ||
			(gx === 7 && gy === 0) ||
			(gx === 7 && gy === 1) ||
			(gx === 0 && gy === 6) ||
			(gx === 0 && gy === 7) ||
			(gx === 1 && gy === 7) ||
			(gx === 7 && gy === 7)
		) {
			return 'void';
		}

		// Back/northeast & central garden terraces that transform from stone into lush foliage
		if (
			(gx === 3 && gy === 1) ||
			(gx === 4 && gy === 1) ||
			(gx === 3 && gy === 2) ||
			(gx === 4 && gy === 2) ||
			(gx === 5 && gy === 2) ||
			(gx === 3 && gy === 3) ||
			(gx === 4 && gy === 3) ||
			(gx === 5 && gy === 3)
		) {
			return 'island_garden';
		}

		// Front/southwest cobblestone promenade + rocky coastal outcrops
		if (
			(gx === 2 && gy === 2) ||
			(gx === 1 && gy === 3) ||
			(gx === 2 && gy === 3) ||
			(gx === 2 && gy === 4) ||
			(gx === 3 && gy === 4) ||
			(gx === 4 && gy === 4) ||
			(gx === 2 && gy === 5) ||
			(gx === 3 && gy === 5)
		) {
			return 'island_stone';
		}

		return 'water';
	}

	/**
	 * Selects the static dark navy water block from Row 8 (col 2..8).
	 * The sprite itself never swaps on a timer—animation comes from gentle vertical bobbing.
	 */
	function getStoneIsleWaterTile(gx: number, gy: number): TileCoord {
		const h = companion.happiness;

		// Underwater blue shoals around the island shore
		if ((gx === 5 && gy === 4) || (gx === 3 && gy === 6)) {
			return { col: 2, row: 8 };
		}
		if (h >= 45 && ((gx === 4 && gy === 5) || (gx === 1 && gy === 4))) {
			return { col: 4, row: 8 };
		}
		if (h >= 70 && ((gx === 6 && gy === 2) || (gx === 2 && gy === 1))) {
			return { col: 2, row: 8 };
		}

		// Subtle dark ripple rings near the island coast
		const nearIsland = gx >= 1 && gx <= 6 && gy >= 1 && gy <= 6;
		if (nearIsland && (gx + gy) % 2 === 0) {
			return { col: 5, row: 8 };
		}

		// Deep dark navy water blocks (col 6, 7, 8, row 8)
		return { col: 6 + ((gx * 3 + gy * 5) % 3), row: 8 };
	}

	/**
	 * Coastal rocks, pebbles in foam rings, water-lilies at higher stages,
	 * and animated twinkling light-blue pixel water sparkles (Row 7 col 5..8).
	 */
	function getStoneIsleWaterOverlay(gx: number, gy: number): TileCoord | null {
		const h = companion.happiness;

		// Small coastal rocks in foam rings tucked right along the SE/SW island shore
		if (gx === 4 && gy === 5) return { col: 4, row: 6 }; // Medium coastal rock in foam
		if (gx === 5 && gy === 4) return { col: 5, row: 6 }; // Twin coastal pebbles in foam
		if (gx === 2 && gy === 6) return { col: 9, row: 6 }; // Tiny single pebble in circular foam ring
		if (gx === 6 && gy === 3 && h >= 45) return { col: 10, row: 6 }; // Emerging eastern stepping stone

		// Floating water-lily blooms appear on the glowing blue shoals at Stage 4 & 5
		if (gx === 3 && gy === 6 && h >= 70) return { col: 2, row: 7 };
		if (gx === 1 && gy === 4 && h >= 90) return { col: 2, row: 7 };

		// Animated twinkling light-blue pixel water sparkles around the island
		if (
			(gx === 1 && gy === 3) ||
			(gx === 1 && gy === 5) ||
			(gx === 2 && gy === 1) ||
			(gx === 5 && gy === 1) ||
			(gx === 6 && gy === 2) ||
			(gx === 6 && gy === 4) ||
			(gx === 5 && gy === 5) ||
			(gx === 4 && gy === 6)
		) {
			const sparkleCol = 5 + ((Math.floor(animFrame / 2) + gx * 3 + gy * 7) % 4);
			return { col: sparkleCol, row: 7 };
		}

		return null;
	}

	/**
	 * Returns the island ground tile for `island_stone` and `island_garden` cells across all 5 stages:
	 * - Stage 1 (0–19%): Pure weathered cobblestone (`col 2, row 6` & `col 6, row 5`)
	 * - Stage 2 (20–44%): Mossy stone blocks (`col 5, 6, row 2`) spread across the northern ridge
	 * - Stage 3 (45–69%): Lush green leafy canopy blocks (`col 7..10, row 2`) cover the central garden
	 * - Stage 4 (70–89%): Deep botanical bush blocks (`col 0..2, row 3`) & mossy stone edges
	 * - Stage 5 (90–100%): Full flourishing island garden with mossy coastal terraces
	 */
	function getStoneIsleLandTile(gx: number, gy: number, kind: StoneIsleCellKind): TileCoord {
		const h = companion.happiness;

		if (kind === 'island_garden') {
			// Core garden cells: (3,2), (4,2), (3,3), (4,3)
			if ((gx === 3 || gx === 4) && (gy === 2 || gy === 3)) {
				if (h >= 70) {
					return (gx + gy) % 2 === 0 ? { col: 9, row: 2 } : { col: 0, row: 3 };
				}
				if (h >= 45) {
					return { col: 7 + ((gx + gy) % 3), row: 2 };
				}
				if (h >= 20) {
					return { col: 5 + ((gx + gy) % 2), row: 2 };
				}
				return gx === 3 && gy === 3 ? { col: 6, row: 5 } : { col: 2, row: 6 };
			}

			// Outer garden ridge: (3,1), (4,1), (5,2), (5,3)
			if (h >= 90) {
				return { col: 8 + ((gx + gy) % 3), row: 2 };
			}
			if (h >= 70) {
				return { col: 7 + ((gx + gy) % 2), row: 2 };
			}
			if (h >= 45) {
				return { col: 5 + ((gx + gy) % 2), row: 2 };
			}
			return { col: 2, row: 6 };
		}

		// Coastal outcrop (1,3) turns mossy at Stage 4+
		if (gx === 1 && gy === 3 && h >= 70) {
			return { col: 5, row: 2 };
		}

		// Inner cobblestone vs water-rimmed shore cobblestone
		const isInner = gx === 3 && gy === 4;
		return isInner ? { col: 6, row: 5 } : { col: 2, row: 6 };
	}

	/**
	 * Rich 5-stage botanical progression on top of the Stone Isle:
	 * - Stage 1 (0–19%): Bare craggy rocks (`col 0, 1, row 6`) and bare driftwood (`col 4, row 4`)
	 * - Stage 2 (20–44%): Hardy green fern tufts (`col 10, row 3`), shrubs (`col 1, row 4`), and mossy logs (`col 6, row 4`)
	 * - Stage 3 (45–69%): First crimson/gold blossoms (`col 9, row 3`), wildflowers (`col 2, row 4`), and sprouting logs (`col 7, row 4`)
	 * - Stage 4 (70–89%): Lush 3-flower orange/red blossom clusters (`col 8, row 3`) & purple lavender bushes (`col 0, row 4`)
	 * - Stage 5 (90–100%): Full Isfahan Oasis bloom across every terrace and walkway border
	 */
	function getStoneIslePlantProp(gx: number, gy: number): TileCoord | null {
		const h = companion.happiness;

		// Walkable cobblestone promenade: stays clear except for delicate border wildflowers at Stage 5 (90%+)
		if (isWalkable(gx, gy)) {
			if (h >= 90 && ((gx === 2 && gy === 2) || (gx === 4 && gy === 4))) {
				return { col: 3, row: 4 };
			}
			return null;
		}

		// Western rocky point (1, 3)
		if (gx === 1 && gy === 3) {
			if (h >= 90) return { col: 0, row: 4 }; // Purple lavender bush
			if (h >= 70) return { col: 2, row: 4 }; // Wildflower cluster
			if (h >= 45) return { col: 7, row: 4 }; // Sprouting mossy log
			if (h >= 20) return { col: 6, row: 4 }; // Mossy log
			return { col: 4, row: 4 }; // Bare driftwood log
		}

		// Northern ridge (3, 1) & (4, 1)
		if (gx === 3 && gy === 1) {
			if (h >= 70) return { col: 0, row: 4 }; // Purple blossom bush
			if (h >= 45) return { col: 1, row: 4 }; // Lush green shrub
			if (h >= 20) return { col: 10, row: 3 }; // Fern shoots
			return { col: 0, row: 6 }; // Craggy grey spire
		}
		if (gx === 4 && gy === 1) {
			if (h >= 90) return { col: 8, row: 3 }; // 3-flower orange/red blossom cluster
			if (h >= 70) return { col: 9, row: 3 }; // Crimson/gold flower
			if (h >= 45) return { col: 10, row: 3 }; // Fern tuft
			return { col: 1, row: 6 }; // Low grey stone
		}

		// Main central garden patch: (3, 2), (4, 2), (3, 3), (4, 3)
		if (gx === 3 && gy === 2) {
			if (h >= 45) return { col: 8, row: 3 }; // Iconic 3-flower orange/red cluster from reference image
			if (h >= 20) return { col: 10, row: 3 }; // Young green shoots
			return null;
		}
		if (gx === 4 && gy === 2) {
			if (h >= 70) return { col: 8, row: 3 }; // Full orange/red blossom cluster
			if (h >= 45) return { col: 9, row: 3 }; // Single red/gold blossom
			if (h >= 20) return { col: 1, row: 4 }; // Green shrub
			return null;
		}
		if (gx === 3 && gy === 3) {
			if (h >= 70) return { col: 9, row: 3 }; // Red/gold blossom on inner garden edge
			if (h >= 45) return { col: 2, row: 4 }; // Yellow/red wildflowers
			if (h >= 20) return { col: 10, row: 3 }; // Fern shoots
			return null;
		}
		if (gx === 4 && gy === 3) {
			if (h >= 90) return { col: 8, row: 3 }; // Peak Oasis blossom cluster
			if (h >= 70) return { col: 2, row: 4 }; // Wildflowers
			if (h >= 45) return { col: 10, row: 3 }; // Fern tuft
			return null;
		}

		// Eastern garden terrace: (5, 2) & (5, 3)
		if (gx === 5 && gy === 2) {
			if (h >= 90) return { col: 0, row: 4 }; // Purple blossom bush
			if (h >= 70) return { col: 9, row: 3 }; // Red/gold flower
			if (h >= 45) return { col: 10, row: 3 }; // Fern shoots
			return { col: 9, row: 5 }; // Weathered grey boulder
		}
		if (gx === 5 && gy === 3) {
			if (h >= 90) return { col: 9, row: 3 };
			if (h >= 70) return { col: 3, row: 4 };
			if (h >= 20) return { col: 10, row: 3 };
			return null;
		}

		return null;
	}

	function stepCritterSimulation(): void {
		animFrame = (animFrame + 1) % 10000;

		if (companion.selectedMap !== lastMapId) {
			lastMapId = companion.selectedMap;
			if (!isWalkable(critterGx, critterGy)) {
				const fallback = companion.selectedMap === 'stone_isle' ? [2, 4] : [3, 5];
				critterGx = fallback[0];
				critterGy = fallback[1];
				targetGx = fallback[0];
				targetGy = fallback[1];
				moveProgress = 0;
				isMoving = false;
			}
		}

		if (isMoving) {
			const speed = companion.isTypingBurst ? 0.2 : 0.12;
			moveProgress += speed;
			if (moveProgress >= 1) {
				critterGx = targetGx;
				critterGy = targetGy;
				moveProgress = 0;
				isMoving = false;
				idleTicksRemaining = companion.isTypingBurst
					? 16 + Math.floor(Math.random() * 14)
					: 42 + Math.floor(Math.random() * 36);
			}
			return;
		}

		if (idleTicksRemaining > 0) {
			idleTicksRemaining -= companion.isTypingBurst ? 2 : 1;
			return;
		}

		const neighbors: { gx: number; gy: number; dir: IsoDirection }[] = [
			{ gx: critterGx + 1, gy: critterGy, dir: 'SE' },
			{ gx: critterGx - 1, gy: critterGy, dir: 'NW' },
			{ gx: critterGx, gy: critterGy + 1, dir: 'SW' },
			{ gx: critterGx, gy: critterGy - 1, dir: 'NE' }
		];

		const valid = neighbors.filter((n) => isWalkable(n.gx, n.gy));
		if (valid.length > 0) {
			const pick = valid[Math.floor(Math.random() * valid.length)];
			targetGx = pick.gx;
			targetGy = pick.gy;
			critterDir = pick.dir;
			moveProgress = 0;
			isMoving = true;
		} else {
			idleTicksRemaining = 36;
		}
	}

	function drawCritter(ctx: CanvasRenderingContext2D): void {
		const curGx = isMoving ? critterGx + (targetGx - critterGx) * moveProgress : critterGx;
		const curGy = isMoving ? critterGy + (targetGy - critterGy) * moveProgress : critterGy;
		const rawPos = isoToScreen(curGx, curGy, 0);
		const pos = {
			x: companion.selectedMap === 'stone_isle' ? rawPos.x + STONE_ISLE_OFFSET_X : rawPos.x,
			y: companion.selectedMap === 'stone_isle' ? rawPos.y + STONE_ISLE_OFFSET_Y - 2 : rawPos.y
		};

		const critterId = companion.selectedCritter;
		const action: CritterAction = isMoving
			? companion.isTypingBurst
				? 'run'
				: 'walk'
			: 'idle';

		const img = critterImages[critterId][action][critterDir] ?? critterImages[critterId].idle.SE;
		if (!img) return;

		if (critterId === 'stag') {
			const fw = 32;
			const fh = 41;
			const totalFrames = Math.max(1, Math.floor(img.naturalWidth / fw));
			const fIdx = animFrame % totalFrames;
			ctx.drawImage(img, fIdx * fw, 0, fw, fh, pos.x, pos.y - 21, fw, fh);
		} else if (critterId === 'badger') {
			const fw = 42;
			const fh = 32;
			const totalFrames = Math.max(1, Math.floor(img.naturalWidth / fw));
			const fIdx = animFrame % totalFrames;
			ctx.drawImage(img, fIdx * fw, 0, fw, fh, pos.x - 5, pos.y - 12, fw, fh);
		} else if (critterId === 'wolf') {
			const fw = 64;
			const fh = 64;
			const totalFrames = Math.max(1, Math.floor(img.naturalWidth / fw));
			const rowMap: Record<IsoDirection, number> = { SW: 0, SE: 1, NW: 2, NE: 3 };
			const row = rowMap[critterDir];
			// Wolf only has 4 breathing frames in wolf-idle.png, so step them at half rate for a natural breath
			const fIdx = isMoving ? animFrame % totalFrames : Math.floor(animFrame / 2) % totalFrames;
			ctx.drawImage(img, fIdx * fw, row * fh, fw, fh, pos.x - 16, pos.y - 34, fw, fh);
		}
	}

	function drawIsometricWorld(ctx: CanvasRenderingContext2D): void {
		ctx.imageSmoothingEnabled = false;
		ctx.clearRect(0, 0, LOGICAL_W, LOGICAL_H);

		if (!tilesetImg) return;

		const curCritterDepth = Math.round(
			(isMoving ? critterGx + (targetGx - critterGx) * moveProgress : critterGx) +
				(isMoving ? critterGy + (targetGy - critterGy) * moveProgress : critterGy)
		);

		// Painter's Algorithm: render back-to-front by (gx + gy)
		const activeGridSize =
			companion.selectedMap === 'stone_isle' ? STONE_ISLE_GRID_SIZE : GRID_SIZE;

		for (let sum = 0; sum <= (activeGridSize - 1) * 2; sum++) {
			for (let gx = 0; gx < activeGridSize; gx++) {
				const gy = sum - gx;
				if (gy < 0 || gy >= activeGridSize) continue;

				const basePos =
					companion.selectedMap === 'stone_isle'
						? {
								x: isoToScreen(gx, gy, 0).x + STONE_ISLE_OFFSET_X,
								y: isoToScreen(gx, gy, 0).y + STONE_ISLE_OFFSET_Y
							}
						: isoToScreen(gx, gy, 0);

				if (companion.selectedMap === 'stone_isle') {
					const kind = getStoneIsleCellKind(gx, gy);
					if (kind === 'void') continue;

					// Slow, gentle vertical bobbing (-1px, 0px, +1px) without changing water sprites
					const waveOffset = Math.round(Math.sin(animFrame * 0.16 + (gx + gy) * 0.6) * 0.72);

					if (kind === 'water') {
						const waterTile = getStoneIsleWaterTile(gx, gy);
						drawSheetTile(ctx, waterTile.col, waterTile.row, basePos.x, basePos.y + waveOffset);

						const waterOverlay = getStoneIsleWaterOverlay(gx, gy);
						if (waterOverlay) {
							const yOff = waterOverlay.row === 6 ? -2 + waveOffset : waveOffset;
							drawSheetTile(ctx, waterOverlay.col, waterOverlay.row, basePos.x, basePos.y + yOff);
						}
					} else {
						// Draw bobbing water underneath coastal cobblestone so foam rims blend into the sea
						drawSheetTile(ctx, 6, 8, basePos.x, basePos.y + 2 + waveOffset);

						const landTile = getStoneIsleLandTile(gx, gy, kind);
						drawSheetTile(ctx, landTile.col, landTile.row, basePos.x, basePos.y - 2);

						const plant = getStoneIslePlantProp(gx, gy);
						if (plant) {
							drawSheetTile(ctx, plant.col, plant.row, basePos.x, basePos.y - 10);
						}
					}
				} else if (isElevatedSanctuary(gx, gy)) {
					drawSheetTile(ctx, 6, 5, basePos.x, basePos.y);
					const topTile = getGroundTile(gx, gy);
					const topPos = isoToScreen(gx, gy, 1);
					drawSheetTile(ctx, topTile.col, topTile.row, topPos.x, topPos.y);

					const prop = getOverlayProp(gx, gy);
					if (prop) {
						drawSheetTile(ctx, prop.col, prop.row, topPos.x, topPos.y - 8);
					}
				} else if (isPondCell(gx, gy)) {
					drawSheetTile(ctx, 6, 8, basePos.x, basePos.y + 1);

					const waveOffset = Math.round(Math.sin(animFrame * 0.16 + (gx + gy) * 0.6) * 0.72);
					const waterTile = getGroundTile(gx, gy);
					drawSheetTile(ctx, waterTile.col, waterTile.row, basePos.x, basePos.y + waveOffset);

					const prop = getOverlayProp(gx, gy);
					if (prop) {
						drawSheetTile(ctx, prop.col, prop.row, basePos.x, basePos.y - 8 + waveOffset);
					}
				} else {
					const groundTile = getGroundTile(gx, gy);
					drawSheetTile(ctx, groundTile.col, groundTile.row, basePos.x, basePos.y);

					const prop = getOverlayProp(gx, gy);
					if (prop) {
						drawSheetTile(ctx, prop.col, prop.row, basePos.x, basePos.y - 8);
					}
				}
			}

			if (sum === curCritterDepth) {
				drawCritter(ctx);
			}
		}

		// Render atmospheric particles (soft glowing fireflies, drifting petals, pond glints, typing motes)
		updateAndDrawParticles(ctx);
	}

	onMount(() => {
		let timeoutId: ReturnType<typeof setTimeout> | null = null;
		let cachedCtx: CanvasRenderingContext2D | null = null;
		let isDestroyed = false;

		preloadAssets().then(() => {
			if (!canvasEl || isDestroyed) return;
			cachedCtx = canvasEl.getContext('2d');
			if (cachedCtx) drawIsometricWorld(cachedCtx);
		});

		// Adaptive low-CPU loop: 12 FPS (82ms) while actively typing or moving, throttling down to
		// 8 FPS (125ms) when idle, and pausing completely when the screen/document is hidden.
		const scheduleNextTick = () => {
			if (isDestroyed) return;
			const isIdle = !companion.isTypingBurst && Date.now() - companion.lastKeystrokeAt > 15_000;
			const delayMs = isIdle ? 125 : 82;
			timeoutId = setTimeout(tick, delayMs);
		};

		const tick = () => {
			if (isDestroyed) return;
			if (typeof document !== 'undefined' && document.hidden) {
				timeoutId = setTimeout(tick, 500);
				return;
			}

			renderSubTick = (renderSubTick + 1) % 2;
			if (renderSubTick === 0) {
				stepCritterSimulation();
			}

			if (!cachedCtx && canvasEl) {
				cachedCtx = canvasEl.getContext('2d');
			}
			if (cachedCtx) {
				drawIsometricWorld(cachedCtx);
			}

			scheduleNextTick();
		};

		scheduleNextTick();

		return () => {
			isDestroyed = true;
			if (timeoutId) clearTimeout(timeoutId);
		};
	});
</script>

<canvas
	bind:this={canvasEl}
	width={LOGICAL_W}
	height={LOGICAL_H}
	class="world-canvas pixelated"
	aria-label="Isfahan isometric pixel-art sanctuary"
></canvas>

<style>
	.world-canvas {
		width: 384px;
		height: 288px;
		display: block;
		pointer-events: none;
	}
</style>
