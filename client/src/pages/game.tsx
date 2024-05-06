import EventManager from "@lomray/event-manager"
import deepcopy from "deepcopy"
import { useEffect, useMemo } from "react"
import Button from "react-bootstrap/Button"
import Modal from "react-bootstrap/Modal"
import Table from "react-bootstrap/Table"
import { Helmet } from "react-helmet-async"
import { Trans, useTranslation } from "react-i18next"
import type { LoaderFunction } from "react-router"
import { json, useLoaderData, useNavigate } from "react-router-dom"
import { useImmer } from "use-immer"
import { useContext } from "../context"
import type { Game as GameType } from "../entities"
import {
    Game as GameEntity,
    GameEvent,
    GameMode,
    GameState,
    Player,
    PlayerState,
} from "../entities"
import { Events, GameEndedData, GameStartedData, ScoredData } from "../events"
import GameService from "../services/games.service"
import AddLocalPlayerScreen from "./game/add-local-player"
import GameEndScreen from "./game/end-screen"
import HitPlayer from "./game/hit-player"
import GameSettings from "./game/settings"
import SlotSelector from "./game/slot-selector"
import { isSlotCorrect } from "./game/utils"

export const loader: LoaderFunction = async ({
    params,
}): Promise<GameEntity> => {
    let gs = new GameService()

    if (params.gameId !== undefined) {
        let game = await gs.get(params.gameId)

        if (game !== undefined) return game
        throw json({ message: "game id not found", status: 404 })
    }
    throw json({ message: "internal api error", status: 500 })
}

export function Game() {
    let gameService = useMemo(() => new GameService(), [])
    let { user } = useContext()
    let [game, setGame] = useImmer(useLoaderData() as GameEntity)
    let [hitSrc, setHitSrc] = useImmer("")
    let [showHits, setShowHits] = useImmer<boolean[]>([])
    let [gameEndedState, setGameEndedState] = useImmer<GameType | null>(null)
    let [showSettings, setShowSettings] = useImmer<boolean>(false)
    let [showAddPlayer, setShowAddPlayer] = useImmer(false)
    let navigate = useNavigate()
    let { t } = useTranslation()
    let [winner, setWinner] = useImmer<Player | null>(null)

    const canSkip = () => {
        return (
            user !== null &&
            ((game.mode === GameMode.Local &&
                game.players.find((p) => p.turn_player)?.state ===
                    PlayerState.Guessing) ||
                (game.mode !== GameMode.Local &&
                    game.players.find((p) => p.id === user.id)?.state ===
                        PlayerState.Guessing)) &&
            (game.players.find((p) => p.turn_player)?.tokens ?? 0) > 0
        )
    }

    useEffect(() => {
        let eventSource = new EventSource(`/api/games/${game.id}/events`)

        eventSource.addEventListener("change_state", (e) => {
            let ge = GameEvent.parse(JSON.parse(e.data))

            if (ge.state === GameState.Guessing) {
                setHitSrc(`/api/games/${game.id}/hit?key=${Math.random()}`)
            } else if (ge.state === GameState.Open) {
                setHitSrc("")
            } else if (ge.state === GameState.Confirming) {
                let winner: string | null = null
                let winners = (ge.players ?? []).filter((p) =>
                    isSlotCorrect(ge.hit ?? null, p.guess),
                )
                if (winners.length == 1) winner = winners[0].id
                else if (winners.length == 2)
                    winner = winners.find((p) => p.turn_player)?.id ?? null
                EventManager.publish(Events.scored, {
                    winner,
                    players: deepcopy(ge.players ?? []),
                    game_mode: game.mode,
                } satisfies ScoredData)
            }

            setGame((g) => {
                if (ge.state === GameState.Open) {
                    if (ge.winner !== undefined)
                        g.players[
                            g.players.findIndex(
                                (p) => p.id === ge.winner?.id,
                            ) as number
                        ] = ge.winner
                    EventManager.publish(Events.gameEnded, {
                        game: deepcopy(g),
                        winner: ge.winner ?? null,
                    } satisfies GameEndedData)
                } else if (
                    ge.state === GameState.Guessing &&
                    g.state === GameState.Open
                ) {
                    EventManager.publish(Events.gameStarted, {
                        game_id: g.id,
                    } satisfies GameStartedData)
                }

                g.state = ge.state as GameState
                g.hit = ge.hit ?? null

                if (ge.players !== undefined) g.players = ge.players
            })
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

            if (
                ge.players === undefined ||
                ge.players.some((p) => p.id === user?.id) === false
            )
                navigate("/")
        })

        eventSource.addEventListener("guess", (e) => {
            let ge = GameEvent.parse(JSON.parse(e.data))
            setGame((g) => {
                ge.players?.forEach((pe) => {
                    let idx = g.players.findIndex((p) => p.id === pe.id)
                    g.players[idx] = pe
                })
            })

            if (ge.players !== undefined) {
                EventManager.publish(Events.guessed, {
                    player_id: ge.players[0].id,
                    guess: ge.players[0].guess,
                })
            }
        })

        eventSource.addEventListener("skip", (e) => {
            let ge = GameEvent.parse(JSON.parse(e.data))
            setGame((g) => {
                ge.players?.forEach((pe) => {
                    let idx = g.players.findIndex((p) => p.id === pe.id)
                    g.players[idx] = pe
                })
            })

            setHitSrc(`/api/games/${game.id}/hit?key=${Math.random()}`)
        })

        eventSource.addEventListener("update", (e) => {
            let ge = GameEvent.parse(JSON.parse(e.data))
            setGame((g) => {
                if (ge.settings !== undefined) {
                    g.hit_duration = ge.settings.hit_duration ?? g.hit_duration
                    g.start_tokens = ge.settings.start_tokens ?? g.start_tokens
                    g.goal = ge.settings.goal ?? g.goal
                    g.packs = ge.settings.packs ?? g.packs
                }
            })
        })

        let unsubscribeGameEnded = EventManager.subscribe(
            Events.gameEnded,
            (e: GameEndedData) => {
                setGameEndedState(e.game)
                setWinner(e.winner)
            },
        )

        if (game.state !== GameState.Open)
            setHitSrc(`/api/games/${game.id}/hit?key=${Math.random()}`)

        return () => {
            eventSource.close()
            unsubscribeGameEnded()
        }
    }, [])

    useEffect(() => {
        setShowHits(Array.from({ length: game.players.length }, () => false))
    }, [game])

    const canStartOrStopGame = (): boolean => {
        return (
            user !== null &&
            game.players.some((p) => p.id === user.id && p.creator === true) &&
            game.players.length >= 2
        )
    }

    return (
        <>
            <Helmet>
                <title>
                    {game.id.toString() +
                        " - " +
                        t("game", { count: 1 }) +
                        " - Hitster"}
                </title>
            </Helmet>
            <h2 className="h4">
                {t("gameId")}: {game.id}, {t("state")}: {game.state}
            </h2>
            <p>{t("gameActions")}</p>
            <Button
                className="me-2"
                disabled={user === null}
                onClick={async () => {
                    if (game.players.some((p) => p.id === user?.id))
                        await gameService.leave(game.id)
                    else await gameService.join(game.id)
                }}
            >
                {user === null
                    ? t("joinGameNotLoggedIn")
                    : game.players.some((p) => p.id === user.id)
                      ? t("leaveGame")
                      : t("joinGame")}
            </Button>
            {user !== null &&
            game.players.find((p) => p.id === user.id)?.creator === true ? (
                <>
                    <Button
                        className="me-2"
                        disabled={!canStartOrStopGame()}
                        onClick={async () => {
                            if (game.state === GameState.Open)
                                await gameService.start(game.id)
                            else await gameService.stop(game.id)
                        }}
                    >
                        {canStartOrStopGame()
                            ? game.state !== GameState.Open
                                ? t("stopGame")
                                : t("startGame")
                            : t("startGameNotEnoughPlayers")}
                    </Button>
                    <Button
                        className="me-2"
                        disabled={
                            game.state !== GameState.Open ||
                            (game.players.find((p) => p.id === user?.id)
                                ?.creator ?? false) === false
                        }
                        aria-expanded={false}
                        onClick={() => setShowSettings(true)}
                    >
                        {game.state !== GameState.Open
                            ? t("gameSettingsNotOpen")
                            : t("gameSettings")}
                    </Button>
                    <GameSettings
                        show={showSettings}
                        game={game}
                        onHide={() => setShowSettings(false)}
                    />
                </>
            ) : (
                ""
            )}
            <h3 className="h5">{t("player", { count: 2 })}</h3>
            <Table responsive>
                <thead>
                    <tr>
                        <th>{t("name")}</th>
                        <th>{t("token", { count: 2 })}</th>
                        <th>{t("hit", { count: 2 })}</th>
                        {user !== null &&
                        game.players.find((p) => p.id === user.id)?.creator ===
                            true ? (
                            <th>{t("kick")}</th>
                        ) : (
                            ""
                        )}
                    </tr>
                </thead>
                <tbody>
                    {game.players.map((p, i) => {
                        return (
                            <tr>
                                <td>
                                    {p.name +
                                        (p.creator === true
                                            ? " (" + t("creator") + ")"
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
                                    >
                                        {t("hit", { count: 2 }) +
                                            `: ${p.hits.length}`}
                                    </Button>
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
                                            closeLabel={t("close")}
                                        >
                                            <Modal.Title>
                                                {t("hitsForPlayer", {
                                                    player: p.name,
                                                })}
                                            </Modal.Title>
                                        </Modal.Header>
                                        <Modal.Body>
                                            <Table responsive>
                                                <thead>
                                                    <tr>
                                                        <th>{t("artist")}</th>
                                                        <th>{t("title")}</th>
                                                        <th>{t("year")}</th>
                                                        <th>
                                                            {t("pack", {
                                                                count: 1,
                                                            })}
                                                        </th>
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
                                                                    {h.artist}
                                                                </td>
                                                                <td>
                                                                    {h.title}
                                                                </td>
                                                                <td>
                                                                    {h.year}
                                                                </td>
                                                                <td>
                                                                    {h.pack}
                                                                </td>
                                                            </tr>
                                                        ))}
                                                </tbody>
                                            </Table>
                                        </Modal.Body>
                                    </Modal>
                                </td>
                                {user !== null &&
                                game.players.find((p) => p.id === user.id)
                                    ?.creator === true ? (
                                    <td>
                                        <Button
                                            disabled={user?.id === p.id}
                                            onClick={async () =>
                                                await gameService.kickPlayer(
                                                    game.id,
                                                    p.id,
                                                )
                                            }
                                        >
                                            {t("kick")}
                                        </Button>
                                    </td>
                                ) : (
                                    ""
                                )}
                            </tr>
                        )
                    })}
                </tbody>
            </Table>
            <Button
                aria-expanded="false"
                disabled={
                    game.mode !== GameMode.Local ||
                    game.state !== GameState.Open
                }
                onClick={() => setShowAddPlayer(true)}
            >
                {game.mode !== GameMode.Local
                    ? t("addPlayerNotLocalGame")
                    : game.state != GameState.Open
                      ? t("addPlayerNotWaiting")
                      : t("addPlayer")}
            </Button>
            {showAddPlayer ? (
                <AddLocalPlayerScreen
                    show={showAddPlayer}
                    onHide={() => setShowAddPlayer(false)}
                    game={game}
                />
            ) : (
                ""
            )}
            <h2 className="h4">{t("hitHeading")}</h2>
            <p aria-live="polite">
                {game.state === GameState.Open ? (
                    t("hitNoGameRunning")
                ) : game.hit === null ? (
                    t("hitUnknown")
                ) : (
                    <Trans
                        i18nKey="hitRevealed"
                        values={{
                            title: game.hit?.title,
                            artist: game.hit?.artist,
                            year: game.hit?.year,
                            pack: game.hit.pack,
                            player:
                                game.players.find((p) =>
                                    isSlotCorrect(game.hit, p.guess),
                                )?.name ?? t("noone"),
                        }}
                        components={[<b />, <b />, <b />, <b />]}
                    />
                )}
            </p>
            <HitPlayer
                src={hitSrc}
                duration={
                    game.state === GameState.Confirming ? 0 : game.hit_duration
                }
            />
            <Button
                className="me-2"
                disabled={!canSkip()}
                onClick={async () =>
                    await gameService.skip(
                        game.id,
                        game.mode === GameMode.Local
                            ? game.players.find((p) => p.turn_player)?.id
                            : undefined,
                    )
                }
            >
                {canSkip()
                    ? t("skipHit")
                    : user === null
                      ? t("skipHitNotLoggedIn")
                      : game.players.find((p) => p.id === user.id)?.state ===
                          PlayerState.Guessing
                        ? t("skipHitNotGuessing")
                        : game.players.find((p) => p.id === user.id)?.tokens ===
                            0
                          ? t("skipHitNoToken")
                          : t("cannotSkipHit")}
            </Button>
            <SlotSelector game={game} />
            {gameEndedState !== null ? (
                <GameEndScreen
                    game={gameEndedState}
                    show={true}
                    winner={winner}
                    onHide={() => {
                        setGameEndedState(null)
                        setWinner(null)
                    }}
                />
            ) : (
                ""
            )}
        </>
    )
}
