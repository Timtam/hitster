import { useEffect } from "react"
import Button from "react-bootstrap/Button"
import Table from "react-bootstrap/Table"
import { useCookies } from "react-cookie"
import { Helmet } from "react-helmet-async"
import type { LoaderFunction } from "react-router"
import { json, useLoaderData, useNavigate } from "react-router-dom"
import { useImmer } from "use-immer"
import { Game as GameEntity, GameEvent, GameState, Player } from "./entities"
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
    let [cookies] = useCookies()
    let [game, setGame] = useImmer(useLoaderData() as GameEntity)
    let navigate = useNavigate()

    useEffect(() => {
        const eventSource = new EventSource(`/api/games/${game.id}/events`)
        eventSource.onmessage = (e) => {
            let ge = GameEvent.parse(e)

            switch (ge.event) {
                case "change_state": {
                    setGame((g) => {
                        g.state = ge.state as GameState
                    })
                    break
                }
                case "join": {
                    setGame((g) => {
                        g.players = ge.players as Player[]
                    })
                    break
                }
                case "leave": {
                    if (ge.players !== undefined)
                        setGame((g) => {
                            g.players = ge.players as Player[]
                        })
                    else navigate("/")
                    break
                }
            }
        }
        return () => {
            eventSource.close()
        }
    }, [])

    return (
        <>
            <Helmet>
                <title>{`${game.id} - Game - Hitster`}</title>
            </Helmet>
            <h2>
                Game ID: {game.id}, State: {game.state}
            </h2>
            <p>Game Actions:</p>
            <Button disabled={cookies.logged_in === undefined}>
                {cookies === undefined
                    ? "You need to be logged in to participate in a game"
                    : game.players.some((p) => p.id === cookies.logged_in.id)
                      ? "Leave game"
                      : "Join game"}
            </Button>
            <h3>Players</h3>
            <Table responsive>
                <thead>
                    <tr>
                        <th>Name</th>
                    </tr>
                </thead>
                <tbody>
                    {game.players.map((p) => (
                        <tr>
                            <td>{p.name}</td>
                        </tr>
                    ))}
                </tbody>
            </Table>
        </>
    )
}
