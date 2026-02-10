import { JSX } from "react"
import { Game, GameMode, Hit, HitIssue, Player, Slot } from "./entities"

export enum Sfx {
    joinGame,
    leaveGame,
    noInterception,
    payToken,
    playHit,
    popup,
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

export interface GameCreatedData {
    game: Game
}

export interface GameRemovedData {
    game: string
}

export interface HitsProgressUpdateData {
    available: number
    downloading: number
    processing: number
}

export interface IssueCreatedData {
    issue: HitIssue
}

export interface IssueDeletedData {
    hitId: string
    issueId: string
}

export enum Events {
    claimedHit = "Claimed hit",
    downloadStarted = "Download started",
    gameCreated = "Game created",
    gameEnded = "Game ended",
    gameRemoved = "Game removed",
    gameStarted = "Game started",
    guessed = "Guessed",
    hitCreated = "Hit created",
    hitRevealed = "Hit revealed",
    hitsProgressUpdate = "Hits progress update",
    issueCreated = "Issue created",
    issueDeleted = "Issue deleted",
    joinedGame = "Joined",
    leftGame = "Left",
    notification = "Notification",
    playSfx = "Play sfx",
    popup = "Popup",
    scored = "Scored",
    sfxEnded = "Sfx ended",
    skippedHit = "Skipped hit",
    slotSelected = "Slot selected",
    tokenReceived = "Token received",
}
