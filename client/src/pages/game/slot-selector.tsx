import { useEffect } from "react"
import Button from "react-bootstrap/Button"
import ToggleButton from "react-bootstrap/ToggleButton"
import ToggleButtonGroup from "react-bootstrap/ToggleButtonGroup"
import { Trans, useTranslation } from "react-i18next"
import { useNavigate } from "react-router-dom"
import { useImmer } from "use-immer"
import { useContext } from "../../context"
import type { Game } from "../../entities"
import { GameMode, GameState, Player, PlayerState } from "../../entities"
import GameService from "../../services/games.service"

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
                            <ToggleButtonGroup
                                name="selected-slot-none"
                                type="radio"
                                defaultValue="0"
                                value={selectedSlot}
                                onChange={(e) => setSelectedSlot(e)}
                            >
                                <ToggleButton
                                    className="me-2 mb-2 border-0"
                                    id="slot-0"
                                    key="slot-0"
                                    value="0"
                                    type="radio"
                                >
                                    {t("dontIntercept")}
                                </ToggleButton>
                            </ToggleButtonGroup>
                            <br aria-hidden />
                        </>
                    ) : (
                        ""
                    )}
                    <ToggleButtonGroup
                        name="selected-slot"
                        type="radio"
                        defaultValue="0"
                        value={selectedSlot}
                        onChange={(e) => setSelectedSlot(e)}
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
                                    <ToggleButton
                                        className="me-2 mb-2"
                                        value={slot.id.toString()}
                                        id={`slot-${slot.id.toString()}`}
                                        key={`slot-${slot.id.toString()}`}
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
