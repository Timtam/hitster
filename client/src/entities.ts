import { z } from "zod"

export const User = z.object({
    username: z.string(),
    id: z.number(),
})

export type User = z.infer<typeof User>

export enum GameState {
    Open = "Open",
    Guessing = "Guessing",
    Confirming = "Confirming",
}

export const Player = z.object({
    id: z.number(),
    name: z.string(),
})

export type Player = z.infer<typeof Player>

export const Game = z.object({
    id: z.number(),
    creator: Player,
    players: z.array(Player),
    state: z.nativeEnum(GameState),
})

export type Game = z.infer<typeof Game>

export const GamesResponse = z.object({
    games: z.array(Game),
})

export type GamesResponse = z.infer<typeof GamesResponse>

export const GameEvent = z.object({
    event: z.string(),
    state: z.optional(z.nativeEnum(GameState)),
    players: z.optional(z.array(Player)),
})

export type GameEvent = z.infer<typeof GameEvent>
