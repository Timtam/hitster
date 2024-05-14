import { z } from "zod"

export const User = z.object({
    name: z.string(),
    id: z.string(),
    virtual: z.boolean(),
    valid_until: z.coerce.date(),
})

export type User = z.infer<typeof User>

export const Hit = z.object({
    artist: z.string(),
    title: z.string(),
    year: z.number(),
    pack: z.string(),
    belongs_to: z.string(),
})

export type Hit = z.infer<typeof Hit>

export const Slot = z.object({
    from_year: z.number(),
    to_year: z.number(),
    id: z.number(),
})

export type Slot = z.infer<typeof Slot>

export enum GameState {
    Open = "Open",
    Guessing = "Guessing",
    Intercepting = "Intercepting",
    Confirming = "Confirming",
}

export enum GameMode {
    Public = "Public",
    Private = "Private",
    Local = "Local",
}

export enum PlayerState {
    Waiting = "Waiting",
    Guessing = "Guessing",
    Intercepting = "Intercepting",
    Confirming = "Confirming",
}

export const Player = z.object({
    id: z.string(),
    name: z.string(),
    state: z.nativeEnum(PlayerState),
    creator: z.boolean(),
    hits: z.array(Hit),
    tokens: z.number(),
    slots: z.array(Slot),
    turn_player: z.boolean(),
    guess: z.nullable(Slot),
})

export type Player = z.infer<typeof Player>

export const Game = z.object({
    id: z.string(),
    players: z.array(Player),
    state: z.nativeEnum(GameState),
    hit_duration: z.number(),
    start_tokens: z.number(),
    goal: z.number(),
    hit: z.nullable(Hit),
    packs: z.array(z.string()),
    mode: z.nativeEnum(GameMode),
    remember_hits: z.boolean(),
})

export type Game = z.infer<typeof Game>

export const GamesResponse = z.object({
    games: z.array(Game),
})

export type GamesResponse = z.infer<typeof GamesResponse>

export const GameSettings = z.object({
    start_tokens: z.optional(z.number()),
    hit_duration: z.optional(z.number()),
    goal: z.optional(z.number()),
    packs: z.optional(z.array(z.string())),
    remember_hits: z.optional(z.boolean()),
})

export type GameSettings = z.infer<typeof GameSettings>

export const GameEvent = z.object({
    state: z.optional(z.nativeEnum(GameState)),
    players: z.optional(z.array(Player)),
    hit: z.optional(Hit),
    settings: z.optional(GameSettings),
    winner: z.optional(Player),
})

export type GameEvent = z.infer<typeof GameEvent>

export const PacksResponse = z.object({
    packs: z.record(z.string(), z.number()),
})

export type PacksResponse = z.infer<typeof PacksResponse>
