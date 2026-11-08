export type CompanionMood = 'sleepy' | 'curious' | 'content' | 'happy' | 'radiant';

export type CritterId = 'stag' | 'badger' | 'wolf';

export type IsoDirection = 'SE' | 'SW' | 'NE' | 'NW';

export type WorldPropId =
	| 'stone_lantern'
	| 'grass_tufts'
	| 'stepping_stones'
	| 'rose_bushes'
	| 'cypress_sapling'
	| 'cypress_tree'
	| 'mosaic_fountain'
	| 'fireflies'
	| 'book_pedestal';

export interface Quote {
	id: string;
	text: string;
	author: string;
	source: string;
	minHappiness: number;
}

export interface StagePalette {
	ground: string;
	accent: string;
	foliage: string;
	water: string;
}

export interface WorldStage {
	id: string;
	name: string;
	minHappiness: number;
	companionMood: CompanionMood;
	palette: StagePalette;
	unlockedProps: WorldPropId[];
}

export interface TileCoord {
	col: number;
	row: number;
}

export interface CritterOption {
	id: CritterId;
	name: string;
	title: string;
}

export interface SpriteManifest {
	canvas: {
		logicalWidth: number;
		logicalHeight: number;
		tileSize: number;
		scale: number;
	};
	tileset: {
		spriteSheet: string;
		tileSize: number;
		columns: number;
		rows: number;
	};
	critters: CritterOption[];
}
