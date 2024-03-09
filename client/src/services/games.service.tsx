import { GamesResponse, Game } from "../entities"

export default class GameService {
    async getAll(): Promise<Game[]> {
        let res = await fetch("/api/games/", {
            method: "GET",
        })
        return GamesResponse.parse(await res.json()).games
    }
}
