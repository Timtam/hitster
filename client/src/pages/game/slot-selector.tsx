import classNames from "classnames"
import { detect } from "detect-browser"
import { useEffect } from "react"
import Button from "react-bootstrap/Button"
import { Trans, useTranslation } from "react-i18next"
import { useNavigate } from "react-router-dom"
import { useImmer } from "use-immer"
import { useContext } from "../../context"
import type { Game } from "../../entities"
import { GameMode, GameState, Player, PlayerState } from "../../entities"
import GameService from "../../services/games.service"
import "./slot-selector.css"

export default ({ game }: { game: Game }) => {
    const { user } = useContext()
    const [selectedSlot, setSelectedSlot] = useImmer("0")
    const navigate = useNavigate()
    let { t } = useTranslation()

    const actionRequired = (): PlayerState => {
        if (user === null) return PlayerState.Waiting
        return actionPlayer()?.state ?? PlayerState.Waiting
    }

    const actionPlayer = (): Player | null => {
        if (game.state === GameState.Open) return null

        let me = game.players.find((p) => p.id === user?.id)

        if (game.mode !== GameMode.Local)
            return me?.state !== PlayerState.Waiting ? (me ?? null) : null
        else {
            return game.state === GameState.Guessing
                ? (game.players.find((p) => p.turn_player) ?? null)
                : game.state === GameState.Intercepting
                  ? (game.players
                        .concat(game.players)
                        .slice(
                            (game.players.findIndex((p) => p.turn_player) ??
                                -1) + 1,
                        )
                        .find((p) => p.state === PlayerState.Intercepting) ??
                    null)
                  : (game.players.find((p) => p.creator) ?? null)
        }
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
                {actionRequired() === PlayerState.Waiting ? (
                    <Trans
                        i18nKey="waitingForPlayerHeading"
                        values={{
                            count: game.players.filter(
                                (p) => p.state != PlayerState.Waiting,
                            ).length,
                            player: joinString(
                                game.players
                                    .filter(
                                        (p) => p.state != PlayerState.Waiting,
                                    )
                                    .map((p) => p.name),
                            ),
                        }}
                        components={[<b />]}
                    />
                ) : actionRequired() === PlayerState.Guessing ? (
                    game.mode !== GameMode.Local &&
                    actionPlayer()?.id === user?.id ? (
                        t("youGuessHeading")
                    ) : (
                        <Trans
                            i18nKey="otherGuessHeading"
                            values={{
                                player: actionPlayer()?.name,
                            }}
                            components={[<b />]}
                        />
                    )
                ) : actionRequired() === PlayerState.Intercepting ? (
                    game.mode !== GameMode.Local &&
                    actionPlayer()?.id === user?.id ? (
                        t("youInterceptHeading")
                    ) : (
                        <Trans
                            i18nKey="otherInterceptHeading"
                            values={{
                                player: actionPlayer()?.name,
                            }}
                            components={[<b />]}
                        />
                    )
                ) : (
                    <Trans
                        i18nKey="confirmHeading"
                        values={{
                            player: game.players.find((p) => p.turn_player)
                                ?.name,
                        }}
                        components={[<b />]}
                    />
                )}
            </h2>
            {actionRequired() === PlayerState.Confirming ? (
                <>
                    <p>
                        <Trans
                            i18nKey="confirmText"
                            values={{
                                player: game.players.find((p) => p.turn_player)
                                    ?.name,
                            }}
                            components={[<b />]}
                        />
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
                    {actionRequired() === PlayerState.Intercepting ? (
                        <>
                            <div className="btn-group mb-2">
                                <input
                                    className="border-0 btn-check"
                                    id="slot-0"
                                    key="slot-0"
                                    value="0"
                                    type="radio"
                                    checked={selectedSlot === "0"}
                                    onChange={(e) =>
                                        setSelectedSlot(e.target.value)
                                    }
                                />
                                <label
                                    htmlFor="slot-0"
                                    className="btn btn-primary"
                                >
                                    {t("dontIntercept")}
                                </label>
                            </div>
                            <br aria-hidden />
                        </>
                    ) : (
                        ""
                    )}
                    <div
                        className="btn-group btn-group-sm mb-2"
                        data-toggle="buttons"
                    >
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
                                    <>
                                        <input
                                            className="mb-2 btn-check"
                                            value={slot.id.toString()}
                                            key={`slot-${slot.id.toString()}`}
                                            id={`slot-${slot.id.toString()}`}
                                            disabled={
                                                (actionRequired() !==
                                                    PlayerState.Guessing &&
                                                    actionRequired() !==
                                                        PlayerState.Intercepting) ||
                                                game.players.some(
                                                    (p) =>
                                                        p.guess?.id === slot.id,
                                                )
                                            }
                                            type="radio"
                                            checked={
                                                selectedSlot ===
                                                slot.id.toString()
                                            }
                                            onChange={(e) =>
                                                setSelectedSlot(e.target.value)
                                            }
                                            title={
                                                text +
                                                (game.players.some(
                                                    (p) =>
                                                        p.guess?.id === slot.id,
                                                )
                                                    ? " (" +
                                                      game.players.find(
                                                          (p) =>
                                                              p.guess?.id ===
                                                              slot.id,
                                                      )?.name +
                                                      ")"
                                                    : "")
                                            }
                                        />
                                        <label
                                            htmlFor={`slot-${slot.id.toString()}`}
                                            className={classNames(
                                                "btn",
                                                "btn-primary",
                                                {
                                                    "radio-disabled":
                                                        (actionRequired() !==
                                                            PlayerState.Guessing &&
                                                            actionRequired() !==
                                                                PlayerState.Intercepting) ||
                                                        game.players.some(
                                                            (p) =>
                                                                p.guess?.id ===
                                                                slot.id,
                                                        ),
                                                },
                                            )}
                                        >
                                            {detect()?.name === "firefox" ? (
                                                <p className="visually-hidden">
                                                    {text +
                                                        (game.players.some(
                                                            (p) =>
                                                                p.guess?.id ===
                                                                slot.id,
                                                        )
                                                            ? " (" +
                                                              game.players.find(
                                                                  (p) =>
                                                                      p.guess
                                                                          ?.id ===
                                                                      slot.id,
                                                              )?.name +
                                                              ")"
                                                            : "")}
                                                </p>
                                            ) : (
                                                ""
                                            )}
                                        </label>
                                        {slot.to_year !== 0 ? (
                                            <span
                                                className="mx-2 mb-2 align-self-center"
                                                aria-hidden={true}
                                            >
                                                {slot.to_year}
                                            </span>
                                        ) : (
                                            ""
                                        )}
                                    </>
                                )
                            })}
                    </div>
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
                                    game.mode === GameMode.Local
                                        ? actionPlayer()?.id
                                        : undefined,
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
