import { Game, GameMode, Hit, Player, Slot } from "./entities"

export enum Sfx {
    joinGame,
    leaveGame,
    noInterception,
    payToken,
    playHit,
    receiveToken,
    selectSlot,
    slotUnavailable,
    stopHit,
    youClaim,
    youFail,
    youLose,
    youScore,
    youWin,
}

export interface SfxData {
    sfx: Sfx
    pan?: number
}

export type PlaySfxData = SfxData

export type SfxEndedData = SfxData

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

export interface SlotSelectedData {
    slot: Slot | null
    slot_count: number
    from_year: number
    to_year: number
    unavailable: boolean
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
    slotSelected = "Slot selected",
    tokenReceived = "Token received",
}
