import type { LoaderFunction } from "react-router"
import { PublicUser, UserStats } from "../entities"
import GameService from "../services/games.service"

const loader: LoaderFunction = async ({
    params,
}): Promise<[PublicUser, UserStats]> => {
    if (params.gameId === undefined || params.playerId === undefined)
        throw { message: "internal api error", status: 500 }

    const gs = new GameService()
    const player = await gs.getPlayer(params.gameId, params.playerId)
    if (player === undefined)
        throw { message: "player not found in this game", status: 404 }

    const stats = await gs.getPlayerStats(params.gameId, params.playerId)
    return [player, stats]
}

export default loader
