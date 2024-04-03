import { json } from "react-router-dom"
import { Game, GamesResponse } from "../entities"

export default class GameService {
    async getAll(): Promise<Game[]> {
        let res = await fetch("/api/games/", {
            method: "GET",
        })
        return GamesResponse.parse(await res.json()).games
    }

    async get(game_id: number): Promise<Game | undefined> {
        let res = await fetch(`/api/games/${game_id}`, {
            method: "GET",
        })

        if (res.status == 200) return Game.parse(await res.json())
        return undefined
    }

    async join(game_id: number) {
        let res = await fetch(`/api/games/${game_id}/join`, {
            method: "PATCH",
            credentials: "include",
        })

        if (res.status == 200) return
        throw json({ message: (await res.json()).message, status: res.status })
    }

    async leave(game_id: number) {
        let res = await fetch(`/api/games/${game_id}/leave`, {
            method: "PATCH",
            credentials: "include",
        })

        if (res.status == 200) return
        throw json({ message: (await res.json()).message, status: res.status })
    }

    async start(game_id: number) {
        let res = await fetch(`/api/games/${game_id}/start`, {
            method: "PATCH",
            credentials: "include",
        })

        if (res.status == 200) return
        throw json({ message: (await res.json()).message, status: res.status })
    }

    async stop(game_id: number) {
        let res = await fetch(`/api/games/${game_id}/stop`, {
            method: "PATCH",
            credentials: "include",
        })

        if (res.status == 200) return
        throw json({ message: (await res.json()).message, status: res.status })
    }

    async guess(game_id: number, slot_id: number | null) {
        let res = await fetch(`/api/games/${game_id}/guess`, {
            body: JSON.stringify({
                id: slot_id,
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
}
