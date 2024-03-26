import { Helmet } from "react-helmet-async"
import type { LoaderFunction } from "react-router"
import { json, useLoaderData } from "react-router-dom"
import { Game as GameEntity } from "./entities"
import GameService from "./services/games.service"

export const loader: LoaderFunction = async ({
    params,
}): Promise<GameEntity> => {
    let gs = new GameService()

    let gameId = parseInt(params.gameId as string, 10)

    if (!Number.isNaN(gameId)) {
        let game = await gs.get(gameId)

        if (game !== undefined) return game
        throw json({ message: "game id not found", status: 404 })
    }
    throw json({ message: "internal api error", status: 500 })
}

export function Game() {
    let game = useLoaderData() as GameEntity

    return (
        <>
            <Helmet>
                <title>{`Game ${game.id} - Hitster`}</title>
            </Helmet>
            <p>Game ID: {game.id}</p>
        </>
    )
}
