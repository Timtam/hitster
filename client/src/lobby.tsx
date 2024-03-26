import Button from "react-bootstrap/Button"
import Table from "react-bootstrap/Table"
import { useCookies } from "react-cookie"
import { Helmet } from "react-helmet-async"
import { Link, useLoaderData, useNavigate } from "react-router-dom"
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
    let navigate = useNavigate()

    useRevalidateOnInterval({ enabled: true, interval: 5000 })

    const createGame = async () => {
        let res = await fetch("/api/games", {
            method: "POST",
            credentials: "include",
        })

        if (res.status === 201)
            navigate("/game/" + Game.parse(await res.json()).id)
    }

    return (
        <>
            <Helmet>
                <title>Game Lobby - Hitster</title>
            </Helmet>
            <Button
                disabled={cookies.logged_in === undefined}
                onClick={createGame}
            >
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
                                <td>
                                    <Link to={"/game/" + game.id}>
                                        {game.id}
                                    </Link>
                                </td>
                                <td>{game.players.length}</td>
                            </tr>
                        )
                    })}
                </tbody>
            </Table>
        </>
    )
}
