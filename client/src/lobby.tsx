import Button from "react-bootstrap/Button"
import Table from "react-bootstrap/Table"
import { useCookies } from "react-cookie"
import { Helmet } from "react-helmet-async"
import { useLoaderData } from "react-router-dom"
import { Game } from "./entities"
import { useRevalidateOnInterval } from "./hooks"
import GameService from "./services/games.service"

export async function loader(): Promise<Game[]> {
    let gs = new GameService()
    return await gs.getAll()
}

export function Lobby() {
    let [cookies] = useCookies(["logged_in"])
    let games = useLoaderData() as Game[]

    useRevalidateOnInterval({ enabled: true, interval: 5000 })

    return (
        <>
            <Helmet>
                <title>Game Lobby - Hitster</title>
            </Helmet>
            <Button disabled={cookies.logged_in === undefined}>
                {cookies.logged_in === undefined
                    ? "You need to be logged in to create a new game"
                    : "Create new game"}
            </Button>
            <Table responsive>
                <thead>
                    <tr>
                        <th>Game ID</th>
                        <th>Players</th>
                    </tr>
                </thead>
                <tbody>
                    {games.map((game) => {
                        return (
                            <tr>
                                <td>{game.id}</td>
                                <td>{game.players.length}</td>
                            </tr>
                        )
                    })}
                </tbody>
            </Table>
        </>
    )
}
