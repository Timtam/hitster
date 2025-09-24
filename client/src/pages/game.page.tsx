import { Helmet } from "@dr.pogodin/react-helmet"
import EventManager from "@lomray/event-manager"
import {
    bindKeyCombo,
    BrowserKeyComboEvent,
    unbindKeyCombo,
} from "@rwh/keystrokes"
import deepcopy from "deepcopy"
import { detect } from "detect-browser"
import { useCallback, useEffect, useMemo } from "react"
import Button from "react-bootstrap/Button"
import ButtonGroup from "react-bootstrap/ButtonGroup"
import Dropdown from "react-bootstrap/Dropdown"
import DropdownButton from "react-bootstrap/DropdownButton"
import Table from "react-bootstrap/Table"
import { Trans, useTranslation } from "react-i18next"
import { Link, useLoaderData, useNavigate } from "react-router"
import { titleCase } from "title-case"
import { useImmer } from "use-immer"
import { useContext } from "../context"
import {
    Game as GameEntity,
    GameEvent,
    GameMode,
    GameState,
    Hit,
    Player,
    PlayerState,
} from "../entities"
import {
    ClaimedHitData,
    Events,
    GameEndedData,
    GameStartedData,
    GuessedData,
    HitRevealedData,
    JoinedGameData,
    LeftGameData,
    ScoredData,
    SkippedHitData,
    TokenReceivedData,
} from "../events"
import FA from "../focus-anchor"
import { useModalShown } from "../hooks"
import GameService from "../services/games.service"
import AddLocalPlayerScreen from "./game/add-local-player"
import GameEndScreen from "./game/end-screen"
import { HitPlayer } from "./game/hit-player"
import HitsView from "./game/hits-view"
import LeaveGameQuestion from "./game/leave-game-question"
import GameSettings from "./game/settings"
import SlotSelector from "./game/slot-selector"

export default function Game() {
    const gameService = useMemo(() => new GameService(), [])
    const { user, showError } = useContext()
    const [game, setGame] = useImmer(useLoaderData() as GameEntity)
    const [hitSrc, setHitSrc] = useImmer("")
    const [showHits, setShowHits] = useImmer<boolean[]>([])
    const [gameEndedState, setGameEndedState] = useImmer<GameEntity | null>(
        null,
    )
    const [showSettings, setShowSettings] = useImmer<boolean>(false)
    const [showAddPlayer, setShowAddPlayer] = useImmer(false)
    const navigate = useNavigate()
    const { t } = useTranslation()
    const [winner, setWinner] = useImmer<Player | null>(null)
    const modalShown = useModalShown()
    const [showLeaveGameQuestion, setShowLeaveGameQuestion] = useImmer(false)

    const joinOrLeaveGame = useCallback(async () => {
        if (game.players.some((p) => p.id === user?.id))
            if (game.state === "Open") await gameService.leave(game.id)
            else setShowLeaveGameQuestion(true)
        else await gameService.join(game.id)
    }, [
        game.id,
        game.players,
        game.state,
        gameService,
        setShowLeaveGameQuestion,
        user,
    ])

    const startOrStopGame = useCallback(async () => {
        if (game.state === GameState.Open) {
            try {
                await gameService.start(game.id)
            } catch (e) {
                showError(
                    (
                        e as {
                            message: string
                            status: number
                        }
                    ).message,
                )
            }
        } else {
            await gameService.stop(game.id)
        }
    }, [game.id, game.state, gameService, showError])

    const canSkip = useCallback(() => {
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
    }, [game.mode, game.players, user])

    const canClaim = (p?: Player) => {
        return p && p.tokens >= 3
    }

    const skipHit = useCallback(
        async () =>
            await gameService.skip(
                game.id,
                game.mode === GameMode.Local
                    ? game.players.find((p) => p.turn_player)?.id
                    : undefined,
            ),
        [game.id, game.mode, game.players, gameService],
    )

    const canStartOrStopGame = useCallback((): boolean => {
        return (
            game.players.find((p) => p.id === user?.id)?.creator === true &&
            game.players.length >= 2
        )
    }, [game.players, user])

    useEffect(() => {
        const eventSource = new EventSource(`/api/games/${game.id}/events`)

        eventSource.addEventListener("change_state", (e) => {
            const ge = GameEvent.parse(JSON.parse(e.data))

            if (ge.state === GameState.Guessing) {
                setHitSrc(`/api/games/${game.id}/hit?key=${Math.random()}`)
            } else if (ge.state === GameState.Open) {
                setHitSrc("")
            } else if (ge.state === GameState.Confirming) {
                EventManager.publish(Events.scored, {
                    winner: ge.last_scored?.id ?? null,
                    players: deepcopy(ge.players ?? []),
                    game_mode: game.mode,
                } satisfies ScoredData)
            }

            if (ge.hit)
                EventManager.publish(Events.hitRevealed, {
                    hit: ge.hit,
                    player: ge.last_scored ?? null,
                } satisfies HitRevealedData)

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
                } else if (ge.state === GameState.Guessing) {
                    if (g.state === GameState.Open) {
                        EventManager.publish(Events.gameStarted, {
                            game_id: g.id,
                        } satisfies GameStartedData)
                    } else if (g.state === GameState.Confirming) {
                        const turn_player = g.players.find((p) => p.turn_player)

                        if (
                            turn_player?.tokens !==
                            (ge.players ?? []).find(
                                (p) => p.id === turn_player?.id,
                            )?.tokens
                        ) {
                            EventManager.publish(Events.tokenReceived, {
                                player: turn_player as Player,
                                game_mode: g.mode,
                            } satisfies TokenReceivedData)
                        }
                    }
                }

                g.last_scored = ge.last_scored ?? null
                g.state = ge.state as GameState
                g.hit = ge.hit ?? null

                if (ge.players !== undefined) g.players = ge.players
            })
        })

        eventSource.addEventListener("join", (e) => {
            const ge = GameEvent.parse(JSON.parse(e.data))
            EventManager.publish(Events.joinedGame, {
                player: (ge.players as Player[])[0],
            } satisfies JoinedGameData)
            setGame((g) => {
                g.players.push((ge.players as Player[])[0])
            })
        })

        eventSource.addEventListener("leave", (e) => {
            const ge = GameEvent.parse(JSON.parse(e.data))
            EventManager.publish(Events.leftGame, {
                player: (ge.players as Player[])[0],
            } satisfies LeftGameData)
            setGame((g) => {
                g.players.splice(
                    g.players.findIndex(
                        (p) => p.id === (ge.players as Player[])[0].id,
                    ),
                    1,
                )

                if (g.players.filter((p) => !p.virtual).length === 0)
                    navigate("/")
            })
        })

        eventSource.addEventListener("guess", (e) => {
            const ge = GameEvent.parse(JSON.parse(e.data))
            setGame((g) => {
                ge.players?.forEach((pe) => {
                    const idx = g.players.findIndex((p) => p.id === pe.id)
                    g.players[idx] = pe
                })
            })

            if (ge.players !== undefined) {
                EventManager.publish(Events.guessed, {
                    player: ge.players[0],
                } satisfies GuessedData)
            }
        })

        eventSource.addEventListener("skip", (e) => {
            const ge = GameEvent.parse(JSON.parse(e.data))
            EventManager.publish(Events.skippedHit, {
                player: (ge.players as Player[])[0],
                hit: ge.hit as Hit,
            } satisfies SkippedHitData)
            setGame((g) => {
                ge.players?.forEach((pe) => {
                    const idx = g.players.findIndex((p) => p.id === pe.id)
                    g.players[idx] = pe
                })
            })

            setHitSrc(`/api/games/${game.id}/hit?key=${Math.random()}`)
        })

        eventSource.addEventListener("claim", (e) => {
            const ge = GameEvent.parse(JSON.parse(e.data))
            EventManager.publish(Events.claimedHit, {
                player: (ge.players as Player[])[0],
                hit: ge.hit as Hit,
                game_mode: game.mode,
            } satisfies ClaimedHitData)
            setGame((g) => {
                ge.players?.forEach((pe) => {
                    const idx = g.players.findIndex((p) => p.id === pe.id)
                    g.players[idx] = pe
                })
            })
        })

        eventSource.addEventListener("update", (e) => {
            const ge = GameEvent.parse(JSON.parse(e.data))
            setGame((g) => {
                if (ge.settings !== undefined) {
                    g.hit_duration = ge.settings.hit_duration ?? g.hit_duration
                    g.start_tokens = ge.settings.start_tokens ?? g.start_tokens
                    g.goal = ge.settings.goal ?? g.goal
                    g.packs = ge.settings.packs ?? g.packs
                }
            })
        })

        const unsubscribeGameEnded = EventManager.subscribe(
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

        // this is by way no good practice, please don't do this at home kids
        // we're waiting for useEffectEvent to become stable
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [navigate, setGame, setGameEndedState, setHitSrc, setWinner])

    useEffect(() => {
        setShowHits(Array.from({ length: game.players.length }, () => false))
    }, [game.players, setShowHits])

    // register keystrokes
    useEffect(() => {
        const handleJoinGame = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()
                joinOrLeaveGame()
            },
        }
        const handleLeaveGame = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()
                joinOrLeaveGame()
            },
        }
        const handleStartOrStopGame = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()
                if (canStartOrStopGame()) startOrStopGame()
            },
        }
        const handleShowSettings = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()
                setShowSettings(true)
            },
        }
        const handleSkipHit = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()
                if (canSkip()) skipHit()
            },
        }

        if (!modalShown) {
            if (game.players.some((p) => p.id === user?.id))
                bindKeyCombo("alt + shift + q", handleLeaveGame)
            else bindKeyCombo("alt + shift + j", handleJoinGame)

            bindKeyCombo("alt + shift + s", handleStartOrStopGame)
            bindKeyCombo("alt + shift + i", handleSkipHit)
            bindKeyCombo("alt + shift + e", handleShowSettings)
        }

        return () => {
            unbindKeyCombo("alt + shift + j", handleJoinGame)
            unbindKeyCombo("alt + shift + q", handleLeaveGame)
            unbindKeyCombo("alt + shift + e", handleShowSettings)
            unbindKeyCombo("alt + shift + s", handleStartOrStopGame)
            unbindKeyCombo("alt + shift + i", handleSkipHit)
        }
    }, [
        canSkip,
        canStartOrStopGame,
        game.state,
        game.players,
        joinOrLeaveGame,
        modalShown,
        setShowSettings,
        skipHit,
        startOrStopGame,
        user,
    ])

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
            <FA>
                <h2 className="h4">
                    {t("gameId")}: {game.id}, {t("state")}: {game.state}
                </h2>
            </FA>
            <LeaveGameQuestion
                show={showLeaveGameQuestion}
                onHide={(yes: boolean) => {
                    if (yes) (async () => await gameService.leave(game.id))()
                    setShowLeaveGameQuestion(false)
                }}
            />
            <p>{t("gameActions")}</p>
            <Button
                className="me-2"
                disabled={
                    game.state !== GameState.Open &&
                    !game.players.some((p) => p.id === user?.id)
                }
                onClick={joinOrLeaveGame}
                aria-keyshortcuts={
                    game.players.some((p) => p.id === user?.id)
                        ? t("leaveGameShortcut")
                        : t("joinGameShortcut")
                }
            >
                {game.players.some((p) => p.id === user?.id)
                    ? t("leaveGame")
                    : t("joinGame")}
            </Button>
            {game.players.find((p) => p.id === user?.id)?.creator ? (
                <>
                    <Button
                        className="me-2"
                        disabled={!canStartOrStopGame()}
                        onClick={startOrStopGame}
                        aria-keyshortcuts={
                            canStartOrStopGame()
                                ? game.state !== GameState.Open
                                    ? t("stopGameShortcut")
                                    : t("startGameShortcut")
                                : ""
                        }
                        aria-label={
                            detect()?.name === "firefox" && canStartOrStopGame()
                                ? game.state !== GameState.Open
                                    ? `${t("stopGameShortcut")} ${t("stopGame")}`
                                    : `${t("startGameShortcut")} ${t("startGame")}`
                                : ""
                        }
                    >
                        {canStartOrStopGame()
                            ? game.state !== GameState.Open
                                ? t("stopGame")
                                : t("startGame")
                            : t("startGameNotEnoughPlayers")}
                    </Button>
                </>
            ) : (
                ""
            )}
            <Button
                className="me-2"
                aria-expanded={false}
                aria-keyshortcuts={t("gameSettingsShortcut")}
                aria-label={
                    detect()?.name === "firefox"
                        ? `${t("gameSettingsShortcut")} ${t("gameSettings")}`
                        : ""
                }
                onClick={() => setShowSettings(true)}
            >
                {t("gameSettings")}
            </Button>
            <GameSettings
                show={showSettings}
                game={game}
                editable={
                    game.state === GameState.Open &&
                    (game.players.find((p) => p.id === user?.id)?.creator ??
                        false) === true
                }
                onHide={() => setShowSettings(false)}
            />
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
                            <tr key={`player-${p.id}`}>
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
                                    <HitsView
                                        show={showHits[i]}
                                        onHide={() =>
                                            setShowHits((h) => {
                                                h[i] = false
                                            })
                                        }
                                        player={p}
                                        gameId={game.id}
                                    />
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
            <p>
                {game.state === GameState.Open ? (
                    t("hitNoGameRunning")
                ) : game.hit === null ? (
                    t("hitUnknown")
                ) : game.hit?.belongs_to === "" ? (
                    <Trans
                        i18nKey="hitRevealed"
                        values={{
                            title: game.hit.title,
                            artist: game.hit.artist,
                            year: game.hit.year,
                            player: titleCase(
                                game.last_scored?.name ?? t("noone"),
                            ),
                        }}
                        components={[
                            <b />,
                            <Link
                                to={`/hits/${game.hit.id}`}
                                target="_blank"
                                rel="noopener noreferrer"
                            >
                                <b />
                            </Link>,
                            <b />,
                            <b />,
                        ]}
                        shouldUnescape={true}
                        tOptions={{ interpolation: { escapeValue: true } }}
                    />
                ) : (
                    <Trans
                        i18nKey="hitRevealedBelonging"
                        values={{
                            title: game.hit.title,
                            artist: game.hit.artist,
                            year: game.hit.year,
                            belongs_to: game.hit.belongs_to,
                            player: titleCase(
                                game.last_scored?.name ?? t("noone"),
                            ),
                        }}
                        components={[
                            <b />,
                            <Link
                                to={`/hits/${game.hit.id}`}
                                target="_blank"
                                rel="noopener noreferrer"
                            >
                                <b />
                            </Link>,
                            <b />,
                            <b />,
                            <b />,
                        ]}
                        shouldUnescape={true}
                        tOptions={{ interpolation: { escapeValue: true } }}
                    />
                )}
            </p>
            <HitPlayer
                src={hitSrc}
                duration={
                    game.state === GameState.Confirming ? 0 : game.hit_duration
                }
                shortcut={t("playOrStopHitShortcut")}
            />
            <Button
                className="me-2"
                disabled={!canSkip()}
                aria-keyshortcuts={canSkip() ? t("skipHitShortcut") : ""}
                aria-label={
                    detect()?.name === "firefox"
                        ? canSkip()
                            ? `${t("skipHitShortcut")} ${t("skipHit")}`
                            : ""
                        : ""
                }
                onClick={skipHit}
            >
                {canSkip()
                    ? t("skipHit")
                    : game.players.find((p) => p.id === user?.id)?.state ===
                        PlayerState.Guessing
                      ? t("skipHitNotGuessing")
                      : game.players.find((p) => p.id === user?.id)?.tokens ===
                          0
                        ? t("skipHitNoToken")
                        : t("cannotSkipHit")}
            </Button>
            {game.mode !== GameMode.Local ? (
                <Button
                    className="me-2"
                    disabled={
                        !canClaim(game.players.find((p) => p.id === user?.id))
                    }
                    onClick={async () =>
                        await gameService.claim(game.id, undefined)
                    }
                >
                    {canClaim(game.players.find((p) => p.id === user?.id))
                        ? t("claimHit")
                        : (game.players.find((p) => p.id === user?.id)
                                ?.tokens ?? 0) < 3
                          ? t("claimHitNoToken")
                          : t("cannotClaimHit")}
                </Button>
            ) : (
                <DropdownButton
                    as={ButtonGroup}
                    title={
                        game.players.every((p) => canClaim(p))
                            ? t("claimHit")
                            : t("cannotClaimHit")
                    }
                    className="me-2"
                    disabled={game.players.every((p) => !canClaim(p))}
                >
                    {game.players.map((p) => (
                        <Dropdown.Item
                            as="button"
                            eventKey={p.id}
                            key={p.id}
                            disabled={!canClaim(p)}
                            onClick={async () =>
                                await gameService.claim(game.id, p.id)
                            }
                        >
                            {p.name}
                        </Dropdown.Item>
                    ))}
                </DropdownButton>
            )}
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
