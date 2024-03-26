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

export const Game = z.object({
    id: z.number(),
    creator: User,
    players: z.array(User),
    state: z.nativeEnum(GameState),
})

export type Game = z.infer<typeof Game>

export const GamesResponse = z.object({
    games: z.array(Game),
})

export type GamesResponse = z.infer<typeof GamesResponse>
