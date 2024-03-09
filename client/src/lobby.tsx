import { Helmet } from "react-helmet-async"
import { Game } from "./entities"
import GameService from "./services/games.service"
import { useLoaderData } from "react-router-dom"

export async function loader(): Promise<Game[]> {
    let gs = new GameService()
    return await gs.getAll()
}

export function Lobby() {
    let games = useLoaderData() as Game[]

    return (
        <>
            <Helmet>
                <title>Game Lobby - Hitster</title>
            </Helmet>
            {games.length
                ? "Voilla, we found some games!"
                : "Nope, no games..."}
        </>
    )
}
