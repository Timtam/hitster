import { Game, GameMode, Hit, Player } from "./entities"

export enum Sfx {
    joinGame,
    leaveGame,
    noInterception,
    payToken,
    playHit,
    receiveToken,
    stopHit,
    youClaim,
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

export interface NotificationData {
    text: string | JSX.Element
    interruptTts?: boolean
    toast?: boolean
}

export interface JoinedGameData {
    player: Player | null
}

export interface LeftGameData {
    player: Player
}

export interface SkippedHitData {
    hit: Hit
    player: Player
}

export interface ClaimedHitData {
    hit: Hit
    player: Player
    game_mode: GameMode
}

export interface HitRevealedData {
    hit: Hit
    player: Player | null
}

export interface TokenReceivedData {
    player: Player
    game_mode: GameMode
}

export enum Events {
    claimedHit = "Claimed hit",
    gameEnded = "Game ended",
    gameStarted = "Game started",
    guessed = "Guessed",
    hitRevealed = "Hit revealed",
    joinedGame = "Joined",
    leftGame = "Left",
    notification = "Notification",
    playSfx = "Play sfx",
    scored = "Scored",
    sfxEnded = "Sfx ended",
    skippedHit = "Skipped hit",
    tokenReceived = "Token received",
}
