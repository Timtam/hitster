import { useEffect, useMemo } from "react"
import Button from "react-bootstrap/Button"
import Modal from "react-bootstrap/Modal"
import Table from "react-bootstrap/Table"
import ToggleButton from "react-bootstrap/ToggleButton"
import ToggleButtonGroup from "react-bootstrap/ToggleButtonGroup"
import { useCookies } from "react-cookie"
import { Helmet } from "react-helmet-async"
import type { LoaderFunction } from "react-router"
import { json, useLoaderData, useNavigate } from "react-router-dom"
import { useImmer } from "use-immer"
import type { Game as GameType } from "./entities"
import {
    Game as GameEntity,
    GameEvent,
    GameState,
    Player,
    PlayerState,
} from "./entities"
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

const SlotSelector = ({ game }: { game: GameType }) => {
    const [selectedSlot, setSelectedSlot] = useImmer(0)
    const [cookies] = useCookies(["logged_in"])
    const navigate = useNavigate()
    const actionRequired = (): PlayerState => {
        if (cookies.logged_in === undefined) return PlayerState.Waiting
        return (
            game.players.find((p) => p.id === cookies.logged_in.id)?.state ??
            PlayerState.Waiting
        )
    }

    const joinString = (parts: string[]): string => {
        if (parts.length === 1) return parts[0]
        else if (parts.length === 2) return parts.join(" and ")
        else
            return (
                parts.slice(0, -1).join(", ") +
                " and " +
                parts[parts.length - 1]
            )
    }

    useEffect(() => {
        game.players.forEach((p) => {
            if (p.guess?.id === selectedSlot) {
                setSelectedSlot(0)
                navigate("", { replace: true })
            }
        })
    }, [game])

    if (game.state === GameState.Open)
        return <h2>The game hasn't started yet.</h2>

    return (
        <>
            <h2>
                {actionRequired() === PlayerState.Waiting
                    ? "You are waiting for " +
                      joinString(
                          game.players
                              .filter((p) => p.state != PlayerState.Waiting)
                              .map((p) => p.name),
                      ) +
                      " to make their move."
                    : actionRequired() === PlayerState.Guessing
                      ? "Finally, its your turn to guess!"
                      : actionRequired() === PlayerState.Intercepting
                        ? "You can now step in and make another guess, but be aware, it'll cost you one token!"
                        : "You now need to confirm if " +
                          game.players.find((p) => p.turn_player)?.name +
                          " guessed title and interpret of the song correctly. Be fair!"}
            </h2>
            <p>
                {actionRequired() === PlayerState.Guessing ||
                actionRequired() === PlayerState.Intercepting
                    ? "Do you think this hit belongs..."
                    : "These are the chosen slots:"}
            </p>
            <ToggleButtonGroup
                name="selected-slot"
                type="radio"
                defaultValue="0"
                onChange={(e) => setSelectedSlot(parseInt(e, 10))}
            >
                {actionRequired() === PlayerState.Intercepting ? (
                    <ToggleButton
                        id="0"
                        value="0"
                        checked={selectedSlot === 0}
                        type="radio"
                    >
                        Don't intercept
                    </ToggleButton>
                ) : (
                    ""
                )}
                {game.players
                    .find((p) => p.turn_player === true)
                    ?.slots.map((slot) => {
                        let text = ""

                        if (slot.from_year === 0)
                            text = `before ${slot.to_year}`
                        else if (slot.to_year === 0)
                            text = `after ${slot.from_year}`
                        else
                            text = `between ${slot.from_year} and ${slot.to_year}`

                        return (
                            <ToggleButton
                                id={slot.id.toString()}
                                value={slot.id}
                                disabled={
                                    (actionRequired() !==
                                        PlayerState.Guessing &&
                                        actionRequired() !==
                                            PlayerState.Intercepting) ||
                                    game.players.some(
                                        (p) => p.guess?.id === slot.id,
                                    )
                                }
                                type="radio"
                            >
                                {text +
                                    (game.players.some(
                                        (p) => p.guess?.id === slot.id,
                                    )
                                        ? " (" +
                                          game.players.find(
                                              (p) => p.guess?.id === slot.id,
                                          )?.name +
                                          ")"
                                        : "")}
                            </ToggleButton>
                        )
                    })}
            </ToggleButtonGroup>
            <Button
                disabled={
                    (selectedSlot === 0 &&
                        actionRequired() === PlayerState.Guessing) ||
                    actionRequired() === PlayerState.Waiting
                }
                onClick={async () => {
                    try {
                        let gs = new GameService()
                        await gs.guess(
                            game.id,
                            selectedSlot > 0 ? selectedSlot : null,
                        )
                        setSelectedSlot(0)
                    } catch (e) {
                        console.log(e)
                    }
                }}
            >
                {actionRequired() === PlayerState.Guessing ||
                actionRequired() === PlayerState.Intercepting
                    ? actionRequired() === PlayerState.Intercepting ||
                      selectedSlot > 0
                        ? "Submit guess"
                        : "Select a slot first"
                    : "You cannot submit a guess right now"}
            </Button>
        </>
    )
}

export function Game() {
    let gameService = useMemo(() => new GameService(), [])
    let [cookies] = useCookies()
    let [game, setGame] = useImmer(useLoaderData() as GameEntity)
    let [hitSrc, setHitSrc] = useImmer("")
    let [showHits, setShowHits] = useImmer<boolean[]>([])
    let navigate = useNavigate()

    useEffect(() => {
        let eventSource = new EventSource(`/api/games/${game.id}/events`)

        eventSource.addEventListener("change_state", (e) => {
            let ge = GameEvent.parse(JSON.parse(e.data))

            setGame((g) => {
                g.state = ge.state as GameState
                g.hit = ge.hit ?? null

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

        eventSource.addEventListener("guess", (e) => {
            let ge = GameEvent.parse(JSON.parse(e.data))
            setGame((g) => {
                ge.players?.forEach((pe) => {
                    let idx = g.players.findIndex((p) => p.id === pe.id)
                    g.players[idx] = pe
                })
            })
        })

        return () => {
            eventSource.close()
        }
    }, [])

    useEffect(() => {
        setShowHits(Array.from({ length: 5 }, () => false))
    }, [game])

    const canStartOrStopGame = (): boolean => {
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
                    ) {
                        await gameService.leave(game.id)
                        navigate("/")
                    } else await gameService.join(game.id)
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
                        <th>Hits</th>
                    </tr>
                </thead>
                <tbody>
                    {game.players.map((p, i) => {
                        return (
                            <tr>
                                <td>
                                    {p.name +
                                        (p.creator === true
                                            ? " (creator)"
                                            : "")}
                                </td>
                                <td>{p.tokens}</td>
                                <td>
                                    <Button
                                        disabled={p.hits.length === 0}
                                        aria-expanded="false"
                                        onClick={() =>
                                            setShowHits((h) => {
                                                h[i] = true
                                            })
                                        }
                                    >{`Hits: ${p.hits.length}`}</Button>
                                    <Modal
                                        show={showHits[i]}
                                        onHide={() =>
                                            setShowHits((h) => {
                                                h[i] = false
                                            })
                                        }
                                    >
                                        <Modal.Header
                                            closeButton
                                            closeLabel="Close"
                                        >
                                            <Modal.Title>
                                                {"Hits for " + p.name}
                                            </Modal.Title>
                                        </Modal.Header>
                                        <Modal.Body>
                                            <Table responsive>
                                                <thead>
                                                    <tr>
                                                        <th>Interpret</th>
                                                        <th>Title</th>
                                                        <th>Year</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {p.hits
                                                        .toSorted(
                                                            (a, b) =>
                                                                a.year - b.year,
                                                        )
                                                        .map((h) => (
                                                            <tr>
                                                                <td>
                                                                    {
                                                                        h.interpret
                                                                    }
                                                                </td>
                                                                <td>
                                                                    {h.title}
                                                                </td>
                                                                <td>
                                                                    {h.year}
                                                                </td>
                                                            </tr>
                                                        ))}
                                                </tbody>
                                            </Table>
                                        </Modal.Body>
                                    </Modal>
                                </td>
                            </tr>
                        )
                    })}
                </tbody>
            </Table>
            <h2>What the hit?</h2>
            <p>
                {game.state === GameState.Open
                    ? "No game is currently running, so no hit for you!"
                    : game.hit === null
                      ? "The hit is hit is currently hidden, you'll have to wait for it to be revealed."
                      : "You're currently listening to " +
                        game.hit?.title +
                        " by " +
                        game.hit?.interpret +
                        " from " +
                        game.hit?.year +
                        "."}
            </p>
            <HitPlayer src={hitSrc} duration={game.hit_duration} />
            <SlotSelector game={game} />
        </>
    )
}
