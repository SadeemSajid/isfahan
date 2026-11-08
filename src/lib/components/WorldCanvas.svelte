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
	const WALKABLE_CELLS: [number, number][] = [
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

	function isWalkable(gx: number, gy: number): boolean {
		return WALKABLE_CELLS.some(([wx, wy]) => wx === gx && wy === gy);
	}

	// Autonomous critter state on the isometric grid
	let critterGx = 3;
	let critterGy = 5;
	let targetGx = 3;
	let targetGy = 5;
	let moveProgress = 0;
	let isMoving = false;
	let critterDir: IsoDirection = 'SE';
	let idleTicksRemaining = 48;
	let animFrame = 0;
	let renderSubTick = 0;

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
				vx: (Math.random() - 0.5) * 0.18,
				vy: -0.05 - Math.random() * 0.12,
				phase: Math.random() * Math.PI * 2,
				phaseSpeed: 0.06 + Math.random() * 0.05,
				life: 0,
				maxLife: 160 + Math.random() * 140,
				colorCore: '#fffbe0',
				colorGlow: 'rgba(255, 220, 105, '
			};
		}
		if (kind === 'petal') {
			const isRose = Math.random() > 0.45;
			return {
				kind,
				x: 36 + Math.random() * 160,
				y: 34 + Math.random() * 40,
				vx: 0.12 + Math.random() * 0.14,
				vy: 0.14 + Math.random() * 0.12,
				phase: Math.random() * Math.PI * 2,
				phaseSpeed: 0.08 + Math.random() * 0.04,
				life: 0,
				maxLife: 140 + Math.random() * 100,
				colorCore: isRose ? '#ff9ebb' : '#ffe180',
				colorGlow: isRose ? 'rgba(232, 74, 95, ' : 'rgba(245, 200, 66, '
			};
		}
		if (kind === 'glint') {
			// Shimmering light glints on the 3x3 pond surface
			return {
				kind,
				x: 94 + (Math.random() - 0.5) * 48,
				y: 88 + (Math.random() - 0.5) * 20,
				vx: 0,
				vy: 0,
				phase: 0,
				phaseSpeed: 0.1,
				life: 0,
				maxLife: 50 + Math.random() * 40,
				colorCore: '#ffffff',
				colorGlow: 'rgba(168, 245, 255, '
			};
		}
		// 'mote' — rising inspiration spark around the critter when typing
		return {
			kind: 'mote',
			x: (originX ?? 120) + (Math.random() - 0.5) * 14,
			y: (originY ?? 100) - 8 + (Math.random() - 0.5) * 6,
			vx: (Math.random() - 0.5) * 0.16,
			vy: -0.28 - Math.random() * 0.18,
			phase: Math.random() * Math.PI * 2,
			phaseSpeed: 0.12,
			life: 0,
			maxLife: 38 + Math.random() * 24,
			colorCore: Math.random() > 0.4 ? '#ffe98a' : '#ff8da6',
			colorGlow: 'rgba(255, 180, 120, '
		};
	}

	function updateAndDrawParticles(ctx: CanvasRenderingContext2D): void {
		const h = companion.happiness;

		// Target counts scale organically with world happiness
		const targetFireflies = h >= 75 ? 9 : h >= 40 ? 5 : 2;
		const targetPetals = h >= 65 ? 5 : h >= 35 ? 2 : 0;
		const targetGlints = h >= 20 ? 4 : 0;

		let fireflyCount = 0;
		let petalCount = 0;
		let glintCount = 0;

		for (const p of particles) {
			if (p.kind === 'firefly') fireflyCount++;
			else if (p.kind === 'petal') petalCount++;
			else if (p.kind === 'glint') glintCount++;
		}

		if (fireflyCount < targetFireflies && Math.random() < 0.15) {
			particles.push(spawnAmbientParticle('firefly'));
		}
		if (petalCount < targetPetals && Math.random() < 0.08) {
			particles.push(spawnAmbientParticle('petal'));
		}
		if (glintCount < targetGlints && Math.random() < 0.12) {
			particles.push(spawnAmbientParticle('glint'));
		}

		// Spawn rising inspiration motes around critter during typing bursts
		if (companion.isTypingBurst && Math.random() < 0.45) {
			const curGx = isMoving ? critterGx + (targetGx - critterGx) * moveProgress : critterGx;
			const curGy = isMoving ? critterGy + (targetGy - critterGy) * moveProgress : critterGy;
			const cPos = isoToScreen(curGx, curGy, 0);
			particles.push(spawnAmbientParticle('mote', cPos.x + 16, cPos.y - 8));
		}

		for (let i = particles.length - 1; i >= 0; i--) {
			const p = particles[i];
			p.life += 1;
			if (p.life >= p.maxLife) {
				particles.splice(i, 1);
				continue;
			}

			p.phase += p.phaseSpeed;
			const lifeProgress = p.life / p.maxLife;
			// Smooth fade-in and fade-out envelope
			const envelope = Math.sin(lifeProgress * Math.PI);

			if (p.kind === 'firefly') {
				p.x += p.vx + Math.sin(p.phase) * 0.18;
				p.y += p.vy + Math.cos(p.phase * 0.7) * 0.08;

				const pulse = 0.45 + 0.55 * (0.5 + 0.5 * Math.sin(p.phase * 1.4));
				const alpha = +(envelope * pulse).toFixed(2);
				const px = Math.round(p.x);
				const py = Math.round(p.y);

				// Soft outer lantern glow halo
				ctx.fillStyle = `${p.colorGlow}${(alpha * 0.28).toFixed(2)})`;
				ctx.fillRect(px - 2, py - 1, 5, 3);
				ctx.fillRect(px - 1, py - 2, 3, 5);

				// Inner warm aura
				ctx.fillStyle = `${p.colorGlow}${(alpha * 0.55).toFixed(2)})`;
				ctx.fillRect(px - 1, py - 1, 3, 3);

				// Crisp bright center core
				ctx.save();
				ctx.globalAlpha = alpha;
				ctx.fillStyle = p.colorCore;
				ctx.fillRect(px, py, 1, 1);
				ctx.restore();
			} else if (p.kind === 'petal') {
				// Swaying breeze motion as blossom petals drift downward
				p.x += p.vx + Math.sin(p.phase) * 0.25;
				p.y += p.vy;

				const px = Math.round(p.x);
				const py = Math.round(p.y);
				const alpha = +(envelope * 0.85).toFixed(2);

				ctx.save();
				ctx.globalAlpha = alpha;
				ctx.fillStyle = p.colorCore;
				// Tiny 2-pixel fluttering petal shape that tilts with phase
				ctx.fillRect(px, py, 2, 1);
				if (Math.sin(p.phase) > 0) {
					ctx.fillRect(px + 1, py + 1, 1, 1);
				} else {
					ctx.fillRect(px, py + 1, 1, 1);
				}
				ctx.restore();
			} else if (p.kind === 'glint') {
				const px = Math.round(p.x);
				const py = Math.round(p.y);
				const alpha = +(envelope * 0.75).toFixed(2);

				ctx.fillStyle = `${p.colorGlow}${(alpha * 0.45).toFixed(2)})`;
				ctx.fillRect(px - 1, py, 3, 1);
				ctx.save();
				ctx.globalAlpha = alpha;
				ctx.fillStyle = p.colorCore;
				ctx.fillRect(px, py, 1, 1);
				ctx.restore();
			} else if (p.kind === 'mote') {
				p.x += p.vx + Math.sin(p.phase) * 0.14;
				p.y += p.vy;

				const px = Math.round(p.x);
				const py = Math.round(p.y);
				const alpha = +(envelope * 0.9).toFixed(2);

				ctx.fillStyle = `${p.colorGlow}${(alpha * 0.35).toFixed(2)})`;
				ctx.fillRect(px - 1, py - 1, 3, 3);
				ctx.save();
				ctx.globalAlpha = alpha;
				ctx.fillStyle = p.colorCore;
				ctx.fillRect(px, py, 1, 1);
				ctx.restore();
			}
		}
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

	function stepCritterSimulation(): void {
		animFrame = (animFrame + 1) % 10000;

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
		const pos = isoToScreen(curGx, curGy, 0);

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
		for (let sum = 0; sum <= (GRID_SIZE - 1) * 2; sum++) {
			for (let gx = 0; gx < GRID_SIZE; gx++) {
				const gy = sum - gx;
				if (gy < 0 || gy >= GRID_SIZE) continue;

				const basePos = isoToScreen(gx, gy, 0);

				if (isElevatedSanctuary(gx, gy)) {
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

					const waveOffset = Math.round(Math.sin(animFrame * 0.28 + (gx + gy) * 0.9) * 0.55);
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
		let intervalId: ReturnType<typeof setInterval> | null = null;

		preloadAssets().then(() => {
			if (!canvasEl) return;
			const ctx = canvasEl.getContext('2d');
			if (ctx) drawIsometricWorld(ctx);
		});

		// 24 FPS render loop: particles glide smoothly every 42ms, while sprite frames advance every 3rd tick (~8 FPS)
		const tick = () => {
			renderSubTick = (renderSubTick + 1) % 3;
			if (renderSubTick === 0) {
				stepCritterSimulation();
			}
			if (!canvasEl) return;
			const ctx = canvasEl.getContext('2d');
			if (!ctx) return;
			drawIsometricWorld(ctx);
		};

		intervalId = setInterval(tick, 42);

		return () => {
			if (intervalId) clearInterval(intervalId);
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
