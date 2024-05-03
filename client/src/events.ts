import { Game, Player, Slot } from "./entities"

export enum Sfx {
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
    player_id: string
    guess: Slot | null
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
}

export enum Events {
    gameEnded = "Game ended",
    gameStarted = "Game started",
    guessed = "Guessed",
    playSfx = "Play sfx",
    scored = "Scored",
    sfxEnded = "Sfx ended",
}
