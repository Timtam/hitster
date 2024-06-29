import { Game, GameMode, Hit, Player } from "./entities"

export enum Sfx {
    joinGame,
    leaveGame,
    noInterception,
    payToken,
    playHit,
    stopHit,
    youFail,
    youLose,
    youScore,
    youWin,
}

export interface SfxData {
    sfx: Sfx
}

export interface PlaySfxData extends SfxData {}

export interface SfxEndedData extends SfxData {}

export interface GuessedData {
    player: Player
}

export interface GameStartedData {
    game_id: string
}

export interface GameEndedData {
    game: Game
    winner: Player | null
}

export interface ScoredData {
    winner: string | null
    players: Player[]
    game_mode: GameMode
}

export interface TtsData {
    text: string
    interrupt?: boolean
}

export interface JoinedGameData {
    player: Player | null
}

export interface LeftGameData {
    player: Player | null
}

export interface HitRevealedData {
    hit: Hit
    player: Player | null
}

export enum Events {
    gameEnded = "Game ended",
    gameStarted = "Game started",
    guessed = "Guessed",
    hitRevealed = "Hit revealed",
    joinedGame = "Joined",
    leftGame = "Left",
    playSfx = "Play sfx",
    scored = "Scored",
    sfxEnded = "Sfx ended",
    tts = "TTS",
}
