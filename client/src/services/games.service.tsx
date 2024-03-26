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
}
