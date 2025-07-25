import type { LoaderFunction } from "react-router"
import { Game } from "../entities"
import GameService from "../services/games.service"

const loader: LoaderFunction = async ({ params }): Promise<Game> => {
    const gs = new GameService()

    if (params.gameId !== undefined) {
        const game = await gs.get(params.gameId)

        if (game !== undefined) return game
        throw { message: "game id not found", status: 404 }
    }
    throw { message: "internal api error", status: 500 }
}

export default loader
