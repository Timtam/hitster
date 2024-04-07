import { useEffect, useMemo } from "react"
import Button from "react-bootstrap/Button"
import Modal from "react-bootstrap/Modal"
import Table from "react-bootstrap/Table"
import ToggleButton from "react-bootstrap/ToggleButton"
import ToggleButtonGroup from "react-bootstrap/ToggleButtonGroup"
import { useCookies } from "react-cookie"
import { Helmet } from "react-helmet-async"
import { Trans, useTranslation } from "react-i18next"
import type { LoaderFunction } from "react-router"
import { json, useLoaderData, useNavigate } from "react-router-dom"
import { useImmer } from "use-immer"
import type { Game as GameType, Hit, Slot } from "./entities"
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
    const [selectedSlot, setSelectedSlot] = useImmer("0")
    const [cookies] = useCookies(["logged_in"])
    const navigate = useNavigate()
    let { t } = useTranslation()
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
                " " +
                t("and") +
                " " +
                parts[parts.length - 1]
            )
    }

    const confirm = async (confirm: boolean) => {
        try {
            let gs = new GameService()
            await gs.confirm(game.id, confirm)
        } catch (e) {
            console.log(e)
        }
    }

    useEffect(() => {
        game.players.forEach((p) => {
            if (p.guess?.id === parseInt(selectedSlot, 10)) {
                setSelectedSlot("0")
                navigate("", { replace: true })
            }
        })
    }, [game])

    if (game.state === GameState.Open)
        return <h2 className="h4">{t("gameNotStarted")}</h2>

    return (
        <>
            <h2 className="h4">
                {actionRequired() === PlayerState.Waiting
                    ? t("waitingForPlayerHeading", {
                          count: game.players.filter(
                              (p) => p.state != PlayerState.Waiting,
                          ).length,
                          player: joinString(
                              game.players
                                  .filter((p) => p.state != PlayerState.Waiting)
                                  .map((p) => p.name),
                          ),
                      })
                    : actionRequired() === PlayerState.Guessing
                      ? t("guessHeading")
                      : actionRequired() === PlayerState.Intercepting
                        ? t("interceptHeading")
                        : t("confirmHeading", {
                              player: game.players.find((p) => p.turn_player)
                                  ?.name,
                          })}
            </h2>
            {actionRequired() === PlayerState.Confirming ? (
                <>
                    <p>
                        {t("confirmText", {
                            player: game.players.find((p) => p.turn_player)
                                ?.name,
                        })}
                    </p>
                    <Button
                        className="me-2"
                        onClick={async () => await confirm(false)}
                    >
                        {t("no")}
                    </Button>
                    <Button
                        className="me-2"
                        onClick={async () => await confirm(true)}
                    >
                        {t("yes")}
                    </Button>
                </>
            ) : (
                <>
                    <p>
                        {actionRequired() === PlayerState.Guessing ||
                        actionRequired() === PlayerState.Intercepting
                            ? t("guessText")
                            : t("waitingText")}
                    </p>
                    <ToggleButtonGroup
                        name="selected-slot"
                        type="radio"
                        defaultValue="0"
                        value={selectedSlot}
                        onChange={(e) => setSelectedSlot(e)}
                    >
                        {actionRequired() === PlayerState.Intercepting ? (
                            <ToggleButton
                                className="me-2 mb-2"
                                id="0"
                                value="0"
                                type="radio"
                            >
                                {t("dontIntercept")}
                            </ToggleButton>
                        ) : (
                            ""
                        )}
                        {game.players
                            .find((p) => p.turn_player === true)
                            ?.slots.map((slot) => {
                                let text = ""

                                if (slot.from_year === 0)
                                    text = t("beforeYear", {
                                        year: slot.to_year,
                                    })
                                else if (slot.to_year === 0)
                                    text = t("afterYear", {
                                        year: slot.from_year,
                                    })
                                else
                                    text = t("betweenYears", {
                                        year1: slot.from_year,
                                        year2: slot.to_year,
                                    })

                                return (
                                    <ToggleButton
                                        className="me-2 mb-2"
                                        id={slot.id.toString()}
                                        value={slot.id.toString()}
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
                                                      (p) =>
                                                          p.guess?.id ===
                                                          slot.id,
                                                  )?.name +
                                                  ")"
                                                : "")}
                                    </ToggleButton>
                                )
                            })}
                    </ToggleButtonGroup>
                    <br aria-hidden="true" />
                    <Button
                        disabled={
                            (selectedSlot === "0" &&
                                actionRequired() === PlayerState.Guessing) ||
                            actionRequired() === PlayerState.Waiting
                        }
                        onClick={async () => {
                            try {
                                let gs = new GameService()
                                await gs.guess(
                                    game.id,
                                    parseInt(selectedSlot, 10) > 0
                                        ? parseInt(selectedSlot, 10)
                                        : null,
                                )
                                setSelectedSlot("0")
                            } catch (e) {
                                console.log(e)
                            }
                        }}
                    >
                        {actionRequired() === PlayerState.Guessing ||
                        actionRequired() === PlayerState.Intercepting
                            ? actionRequired() === PlayerState.Intercepting ||
                              parseInt(selectedSlot, 10) > 0
                                ? t("submitGuess")
                                : t("selectSlotFirst")
                            : t("cannotSubmitGuess")}
                    </Button>
                </>
            )}
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
    let { t } = useTranslation()

    const isSlotCorrect = (hit: Hit | null, slot: Slot | null): boolean => {
        if (hit === null || slot === null) return false
        return (
            (slot.from_year === 0 && hit.year <= slot.to_year) ||
            (slot.to_year === 0 && hit.year >= slot.from_year) ||
            (slot.from_year <= hit.year && hit.year <= slot.to_year)
        )
    }

    const canSkip = () => {
        return (
            cookies.logged_in !== undefined &&
            (game.players.find((p) => p.id === cookies.logged_in.id)?.state ??
                PlayerState.Waiting) === PlayerState.Guessing &&
            (game.players.find((p) => p.id === cookies.logged_in.id)?.tokens ??
                0) > 0
        )
    }

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
                setHitSrc(`/api/games/${game.id}/hit?key=${Math.random()}`)
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

        if (game.state !== GameState.Open)
            setHitSrc(`/api/games/${game.id}/hit?key=${Math.random()}`)

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
                {cookies.logged_in === undefined
                    ? t("joinGameNotLoggedIn")
                    : game.players.some((p) => p.id === cookies.logged_in.id)
                      ? t("leaveGame")
                      : t("joinGame")}
            </Button>
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
                    : cookies.logged_in === undefined
                      ? t("startGameNotLoggedIn")
                      : game.players.length < 2
                        ? t("startGameNotEnoughPlayers")
                        : t("startGameNotCreator")}
            </Button>
            <h3 className="h5">{t("player", { count: 2 })}</h3>
            <Table responsive>
                <thead>
                    <tr>
                        <th>{t("name")}</th>
                        <th>{t("token", { count: 2 })}</th>
                        <th>{t("hit", { count: 2 })}</th>
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
                            player:
                                game.players.find((p) =>
                                    isSlotCorrect(game.hit, p.guess),
                                )?.name ?? t("noone"),
                        }}
                        components={[<b />, <b />, <b />]}
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
                onClick={async () => await gameService.skip(game.id)}
            >
                {canSkip()
                    ? t("skipHit")
                    : cookies.logged_in === undefined
                      ? t("skipHitNotLoggedIn")
                      : game.players.find((p) => p.id === cookies.logged_in.id)
                              ?.state === PlayerState.Guessing
                        ? t("skipHitNotGuessing")
                        : game.players.find(
                                (p) => p.id === cookies.logged_in.id,
                            )?.tokens === 0
                          ? t("skipHitNoToken")
                          : t("cannotSkipHit")}
            </Button>
            <SlotSelector game={game} />
        </>
    )
}
