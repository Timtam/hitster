import { z } from "zod"

function createPaginatedResponseSchema<ItemType extends z.ZodTypeAny>(
    itemSchema: ItemType,
) {
    return z.object({
        total: z.number(),
        start: z.number(),
        end: z.number(),
        results: z.array(itemSchema),
    })
}

export const Permissions = z.object({
    write_hits: z.boolean(),
    write_packs: z.boolean(),
    read_issues: z.boolean(),
    write_issues: z.boolean(),
    delete_issues: z.boolean(),
})

export type Permissions = z.infer<typeof Permissions>

export const User = z.object({
    name: z.string(),
    id: z.uuid(),
    virtual: z.boolean(),
    valid_until: z.coerce.date(),
    permissions: Permissions,
})

export type User = z.infer<typeof User>

export enum HitIssueType {
    Auto = "auto",
    Custom = "custom",
}

export const HitIssue = z.object({
    id: z.uuid(),
    hit_id: z.uuid(),
    type: z.nativeEnum(HitIssueType),
    message: z.string(),
    created_at: z.coerce.date(),
    last_modified: z.coerce.date(),
})

export type HitIssue = z.infer<typeof HitIssue>

export const Hit = z.object({
    artist: z.string(),
    title: z.string(),
    year: z.number(),
    packs: z.array(z.string()),
    belongs_to: z.string(),
    id: z.uuid(),
    downloaded: z.optional(z.boolean()),
    issues: z.optional(z.array(HitIssue)),
})

export type Hit = z.infer<typeof Hit>

export const FullHit = z.object({
    artist: z.string(),
    title: z.string(),
    year: z.number(),
    packs: z.array(z.string()),
    belongs_to: z.string(),
    id: z.optional(z.uuid()),
    yt_id: z.string(),
    playback_offset: z.number(),
    downloaded: z.optional(z.boolean()),
    issues: z.optional(z.array(HitIssue)),
})

export type FullHit = z.infer<typeof FullHit>

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
    id: z.uuid(),
    name: z.string(),
    state: z.nativeEnum(PlayerState),
    creator: z.boolean(),
    hits: z.array(Hit),
    tokens: z.number(),
    slots: z.array(Slot),
    turn_player: z.boolean(),
    guess: z.nullable(Slot),
    virtual: z.boolean(),
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
    last_scored: z.nullable(Player),
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
})

export type GameSettings = z.infer<typeof GameSettings>

export const GameEvent = z.object({
    state: z.optional(z.nativeEnum(GameState)),
    players: z.optional(z.array(Player)),
    hit: z.optional(Hit),
    settings: z.optional(GameSettings),
    winner: z.optional(Player),
    last_scored: z.optional(Player),
})

export type GameEvent = z.infer<typeof GameEvent>

export const Pack = z.object({
    id: z.uuid(),
    name: z.string(),
    hits: z.number(),
})

export type Pack = z.infer<typeof Pack>

export const PacksResponse = z.object({
    packs: z.array(Pack),
})

export type PacksResponse = z.infer<typeof PacksResponse>

export const CreateGameEvent = z.object({
    create_game: Game,
})

export type CreateGameEvent = z.infer<typeof CreateGameEvent>

export const RemoveGameEvent = z.object({
    remove_game: z.string(),
})

export type RemoveGameEvent = z.infer<typeof RemoveGameEvent>

export const ProcessHitsEvent = z.object({
    process_hits: z.object({
        available: z.number(),
        downloading: z.number(),
        processing: z.number(),
    }),
})

export type ProcessHitsEvent = z.infer<typeof ProcessHitsEvent>

export const CreateHitIssueEvent = z.object({
    create_hit_issue: HitIssue,
})

export type CreateHitIssueEvent = z.infer<typeof CreateHitIssueEvent>

export const DeleteHitIssueEvent = z.object({
    delete_hit_issue: z.object({
        hit_id: z.uuid(),
        issue_id: z.uuid(),
    }),
})

export type DeleteHitIssueEvent = z.infer<typeof DeleteHitIssueEvent>

export const PaginatedHitsResponse = createPaginatedResponseSchema(Hit)

export type PaginatedHitsResponse = z.infer<typeof PaginatedHitsResponse>

export enum SortBy {
    Title = "title",
    Artist = "artist",
    BelongsTo = "belongs_to",
    Year = "year",
}

export enum SortDirection {
    Ascending = "ascending",
    Descending = "descending",
}

export enum HitQueryPart {
    Issues = "issues",
    Downloaded = "downloaded",
}

export enum HitSearchFilter {
    HasIssues = "has_issues",
}

export const HitSearchQuery = z.object({
    sort_by: z.optional(z.array(z.nativeEnum(SortBy))),
    sort_direction: z.optional(z.nativeEnum(SortDirection)),
    query: z.optional(z.string()),
    packs: z.optional(z.array(z.uuid())),
    start: z.optional(z.number()),
    amount: z.optional(z.number()),
    parts: z.optional(z.array(z.nativeEnum(HitQueryPart))),
    filters: z.optional(z.array(z.nativeEnum(HitSearchFilter))),
})

export type HitSearchQuery = z.infer<typeof HitSearchQuery>
