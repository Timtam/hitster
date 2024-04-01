import { useEffect, useMemo } from "react"
import Button from "react-bootstrap/Button"
import Table from "react-bootstrap/Table"
import { useCookies } from "react-cookie"
import { Helmet } from "react-helmet-async"
import type { LoaderFunction } from "react-router"
import { json, useLoaderData, useNavigate } from "react-router-dom"
import { useImmer } from "use-immer"
import { Game as GameEntity, GameEvent, GameState, Player } from "./entities"
import HitPlayer from "./hit-player"
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
    let gameService = useMemo(() => new GameService(), [])
    let [cookies] = useCookies()
    let [game, setGame] = useImmer(useLoaderData() as GameEntity)
    let [hitSrc, setHitSrc] = useImmer("")
    let navigate = useNavigate()

    useEffect(() => {
        let eventSource = new EventSource(`/api/games/${game.id}/events`)

        eventSource.addEventListener("change_state", (e) => {
            let ge = GameEvent.parse(JSON.parse(e.data))

            setGame((g) => {
                g.state = ge.state as GameState

                if (ge.players !== undefined) g.players = ge.players
            })

            if (ge.state === GameState.Guessing) {
                setHitSrc("")
                setHitSrc(`/api/games/${game.id}/hit`)
            } else if (ge.state === GameState.Open) {
                setHitSrc("")
            }
        })

        eventSource.addEventListener("join", (e) => {
            let ge = GameEvent.parse(JSON.parse(e.data))
            setGame((g) => {
                g.players = ge.players as Player[]
            })
        })

        eventSource.addEventListener("leave", (e) => {
            let ge = GameEvent.parse(JSON.parse(e.data))
            if (ge.players !== undefined)
                setGame((g) => {
                    g.players = ge.players as Player[]
                })
            else navigate("/")
        })
        return () => {
            eventSource.close()
        }
    }, [])

    const canStartOrStopGame = () => {
        return (
            cookies.logged_in !== undefined &&
            game.players.some(
                (p) => p.id === cookies.logged_in.id && p.creator === true,
            ) &&
            game.players.length >= 2
        )
    }

    return (
        <>
            <Helmet>
                <title>{`${game.id} - Game - Hitster`}</title>
            </Helmet>
            <h2>
                Game ID: {game.id}, State: {game.state}
            </h2>
            <p>Game Actions:</p>
            <Button
                disabled={cookies.logged_in === undefined}
                onClick={async () => {
                    if (
                        game.players.some((p) => p.id === cookies.logged_in?.id)
                    )
                        await gameService.leave(game.id)
                    else await gameService.join(game.id)
                }}
            >
                {cookies === undefined
                    ? "You need to be logged in to participate in a game"
                    : game.players.some((p) => p.id === cookies.logged_in.id)
                      ? "Leave game"
                      : "Join game"}
            </Button>
            <Button
                disabled={!canStartOrStopGame()}
                onClick={async () => {
                    if (game.state === GameState.Open)
                        await gameService.start(game.id)
                    else await gameService.stop(game.id)
                }}
            >
                {canStartOrStopGame()
                    ? game.state !== GameState.Open
                        ? "Stop game"
                        : "Start game"
                    : cookies.logged_in === undefined
                      ? "You must be logged in to start or stop a game"
                      : game.players.length < 2
                        ? "At least two players must be part of a game"
                        : "Only the creator can start or stop a game"}
            </Button>
            <h3>Players</h3>
            <Table responsive>
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Tokens</th>
                    </tr>
                </thead>
                <tbody>
                    {game.players.map((p) => (
                        <tr>
                            <td>
                                {p.name +
                                    (p.creator === true ? " (creator)" : "")}
                            </td>
                            <td>{p.tokens}</td>
                        </tr>
                    ))}
                </tbody>
            </Table>
            <HitPlayer src={hitSrc} duration={game.hit_duration} />
        </>
    )
}
