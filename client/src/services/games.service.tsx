import { json } from "react-router-dom"
import type { GameMode, GameSettings } from "../entities"
import { Game, GamesResponse } from "../entities"
import fetchAuth from "../fetch"

export default class GameService {
    async getAll(): Promise<Game[]> {
        let res = await fetch("/api/games/", {
            method: "GET",
        })
        return GamesResponse.parse(await res.json()).games
    }

    async get(game_id: string): Promise<Game | undefined> {
        let res = await fetch(`/api/games/${game_id}`, {
            method: "GET",
        })

        if (res.status == 200) return Game.parse(await res.json())
        return undefined
    }

    async create(mode: GameMode): Promise<Game> {
        let res = await fetchAuth("/api/games", {
            method: "POST",
            credentials: "include",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                mode: mode,
            }),
        })

        if (res.status === 201) return Game.parse(await res.json())
        throw json({ message: (await res.json()).message, status: res.status })
    }

    async join(game_id: string) {
        let res = await fetchAuth(`/api/games/${game_id}/join`, {
            method: "PATCH",
            credentials: "include",
        })

        if (res.status == 200) return
        throw json({ message: (await res.json()).message, status: res.status })
    }

    async leave(game_id: string) {
        let res = await fetchAuth(`/api/games/${game_id}/leave`, {
            method: "PATCH",
            credentials: "include",
        })

        if (res.status == 200) return
        throw json({ message: (await res.json()).message, status: res.status })
    }

    async start(game_id: string) {
        let res = await fetchAuth(`/api/games/${game_id}/start`, {
            method: "PATCH",
            credentials: "include",
        })

        if (res.status == 200) return
        throw json({ message: (await res.json()).message, status: res.status })
    }

    async stop(game_id: string) {
        let res = await fetchAuth(`/api/games/${game_id}/stop`, {
            method: "PATCH",
            credentials: "include",
        })

        if (res.status == 200) return
        throw json({ message: (await res.json()).message, status: res.status })
    }

    async guess(game_id: string, slot_id: number | null, player_id?: string) {
        let res = await fetchAuth(
            `/api/games/${game_id}/guess/${player_id ?? ""}`,
            {
                body: JSON.stringify({
                    id: slot_id,
                }),
                headers: {
                    "Content-Type": "application/json",
                },
                method: "POST",
                credentials: "include",
            },
        )

        if (res.status == 200) return
        throw json({ message: (await res.json()).message, status: res.status })
    }

    async confirm(game_id: string, confirmation: boolean) {
        let res = await fetchAuth(`/api/games/${game_id}/confirm`, {
            body: JSON.stringify({
                confirm: confirmation,
            }),
            headers: {
                "Content-Type": "application/json",
            },
            method: "POST",
            credentials: "include",
        })

        if (res.status == 200) return
        throw json({ message: (await res.json()).message, status: res.status })
    }

    async skip(game_id: string, player_id?: string) {
        let res = await fetchAuth(
            `/api/games/${game_id}/skip/${player_id ?? ""}`,
            {
                method: "POST",
                credentials: "include",
            },
        )

        if (res.status == 200) return
        throw json({ message: (await res.json()).message, status: res.status })
    }

    async update(game_id: string, settings: GameSettings) {
        let res = await fetchAuth(`/api/games/${game_id}/update`, {
            body: JSON.stringify(settings),
            headers: {
                "Content-Type": "application/json",
            },
            method: "PATCH",
            credentials: "include",
        })

        if (res.status == 200) return
        throw json({ message: (await res.json()).message, status: res.status })
    }

    async addPlayer(game_id: string, player_name: string) {
        let res = await fetchAuth(`/api/games/${game_id}/join/${player_name}`, {
            method: "PATCH",
            credentials: "include",
        })

        if (res.status == 200) return
        throw json({ message: (await res.json()).message, status: res.status })
    }

    async kickPlayer(game_id: string, player_id: string) {
        let res = await fetchAuth(`/api/games/${game_id}/leave/${player_id}`, {
            method: "PATCH",
            credentials: "include",
        })

        if (res.status == 200) return
        throw json({ message: (await res.json()).message, status: res.status })
    }
}
