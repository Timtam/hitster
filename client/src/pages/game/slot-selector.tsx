import EventManager from "@lomray/event-manager"
import {
    bindKeyCombo,
    BrowserKeyComboEvent,
    unbindKeyCombo,
} from "@rwh/keystrokes"
import { detect } from "detect-browser"
import { useEffect, useState } from "react"
import Button from "react-bootstrap/Button"
import ToggleButton from "react-bootstrap/ToggleButton"
import ToggleButtonGroup from "react-bootstrap/ToggleButtonGroup"
import { Trans, useTranslation } from "react-i18next"
import { useContext } from "../../context"
import type { Game, Slot } from "../../entities"
import { GameMode, GameState, Player, PlayerState } from "../../entities"
import { Events, NotificationData, SlotSelectedData } from "../../events"
import GameService from "../../services/games.service"

export default ({ game }: { game: Game }) => {
    const { user } = useContext()
    const [selectedSlot, setSelectedSlot] = useState("0")
    const [selectedKeySlot, setSelectedKeySlot] = useState("0")
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

    const guess = async () => {
        if (selectedSlot === selectedKeySlot) {
            try {
                let gs = new GameService()
                await gs.guess(
                    game.id,
                    selectedSlot !== "0" ? parseInt(selectedSlot, 10) : null,
                    game.mode === GameMode.Local
                        ? actionPlayer()?.id
                        : undefined,
                )
                setSelectedSlot("0")
                setSelectedKeySlot("0")
            } catch (e) {
                console.log(e)
            }
        }
    }

    useEffect(() => {
        game.players.forEach((p) => {
            if (p.guess?.id.toString() === selectedSlot) {
                setSelectedSlot("0")
            }
        })

        if (
            selectedSlot === "0" &&
            selectedKeySlot !== "0" &&
            !game.players.some(
                (p) => p.guess?.id.toString() === selectedKeySlot,
            )
        ) {
            setSelectedSlot(selectedKeySlot)
        }
    }, [game, selectedKeySlot, selectedSlot])

    useEffect(() => {
        let handlePreviousSlot = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()
                let slot = "0"
                let p = game.players.find((p) => p.turn_player) as Player

                if (selectedKeySlot === "0" || selectedKeySlot === "1")
                    slot = p.slots.length.toString()
                else slot = (parseInt(selectedKeySlot, 10) - 1).toString()

                let s = p.slots.find((s) => s.id === parseInt(slot, 10)) as Slot
                let u =
                    (actionRequired() !== PlayerState.Guessing &&
                        actionRequired() !== PlayerState.Intercepting) ||
                    game.players.some((p) => p.guess?.id === s.id)

                let text = ""

                if (s.from_year === 0)
                    text = t("beforeYear", {
                        year: s.to_year,
                    })
                else if (s.to_year === 0)
                    text = t("afterYear", {
                        year: s.from_year,
                    })
                else
                    text = t("betweenYears", {
                        year1: s.from_year,
                        year2: s.to_year,
                    })

                EventManager.publish(Events.notification, {
                    toast: false,
                    interruptTts: true,
                    text: text,
                } satisfies NotificationData)

                EventManager.publish(Events.slotSelected, {
                    unavailable: u,
                    slot: s,
                    from_year: p.slots[0].to_year,
                    to_year: p.slots[p.slots.length - 1].from_year,
                    slot_count: p.slots.length,
                } satisfies SlotSelectedData)

                setSelectedKeySlot(slot)
                if (!u) setSelectedSlot(slot)
                else setSelectedSlot("0")
            },
        }

        let handleNextSlot = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()

                let slot = "0"
                let p = game.players.find((p) => p.turn_player) as Player

                if (
                    selectedKeySlot === "0" ||
                    selectedKeySlot === p.slots.length.toString()
                )
                    slot = "1"
                else slot = (parseInt(selectedKeySlot, 10) + 1).toString()

                let s = p.slots.find((s) => s.id === parseInt(slot, 10)) as Slot
                let u =
                    (actionRequired() !== PlayerState.Guessing &&
                        actionRequired() !== PlayerState.Intercepting) ||
                    game.players.some((p) => p.guess?.id === s.id)

                let text = ""

                if (s.from_year === 0)
                    text = t("beforeYear", {
                        year: s.to_year,
                    })
                else if (s.to_year === 0)
                    text = t("afterYear", {
                        year: s.from_year,
                    })
                else
                    text = t("betweenYears", {
                        year1: s.from_year,
                        year2: s.to_year,
                    })

                EventManager.publish(Events.notification, {
                    toast: false,
                    interruptTts: true,
                    text: text,
                } satisfies NotificationData)

                EventManager.publish(Events.slotSelected, {
                    unavailable: u,
                    slot: s,
                    from_year: p.slots[0].to_year,
                    to_year: p.slots[p.slots.length - 1].from_year,
                    slot_count: p.slots.length,
                } satisfies SlotSelectedData)

                setSelectedKeySlot(slot)
                if (!u) setSelectedSlot(slot)
                else setSelectedSlot("0")
            },
        }

        let handleResetSlot = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()
                if (selectedKeySlot !== "0") {
                    setSelectedKeySlot("0")
                    setSelectedSlot("0")

                    let p = game.players.find((p) => p.turn_player) as Player

                    EventManager.publish(Events.slotSelected, {
                        unavailable: true,
                        slot: null,
                        from_year: p.slots[0].to_year,
                        to_year: p.slots[p.slots.length - 1].from_year,
                        slot_count: p.slots.length,
                    } satisfies SlotSelectedData)
                }
            },
        }

        let handleGuess = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()
                guess()
            },
        }

        let handleConfirmYes = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()
                confirm(true)
            },
        }

        let handleConfirmNo = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()
                confirm(false)
            },
        }

        let handleReadPlayerStats = Array.from({ length: 10 }, (_, i) => ({
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()
                if (!game.players[i]) {
                    return
                }

                EventManager.publish(Events.notification, {
                    toast: false,
                    interruptTts: true,
                    text: t("playerStatsNotification", {
                        player: game.players[i].name,
                        hits: game.players[i].hits.length,
                        tokens: game.players[i].tokens,
                    }),
                } satisfies NotificationData)
            },
        }))

        if (game.state !== GameState.Open) {
            for (let i = 0; i < 10; i++) {
                bindKeyCombo(
                    "alt + shift + @Digit" + (i !== 9 ? i + 1 : 0).toString(),
                    handleReadPlayerStats[i],
                )
            }
        }

        if (
            game.state !== GameState.Confirming &&
            game.state !== GameState.Open
        ) {
            bindKeyCombo("alt + shift + ArrowUp", handlePreviousSlot)
            bindKeyCombo("alt + shift + ArrowDown", handleNextSlot)
            bindKeyCombo("alt + shift + Backspace", handleResetSlot)
            bindKeyCombo("alt + shift + Enter", handleGuess)
        } else if (actionRequired() === PlayerState.Confirming) {
            bindKeyCombo("alt + shift + y", handleConfirmYes)
            bindKeyCombo("alt + shift + n", handleConfirmNo)
        }

        return () => {
            unbindKeyCombo("alt + shift + ArrowUp", handlePreviousSlot)
            unbindKeyCombo("alt + shift + ArrowDown", handleNextSlot)
            unbindKeyCombo("alt + shift + Backspace", handleResetSlot)
            unbindKeyCombo("alt + shift + Enter", handleGuess)
            unbindKeyCombo("alt + shift + y", handleConfirmYes)
            unbindKeyCombo("alt + shift + n", handleConfirmNo)

            for (let i = 0; i < 10; i++) {
                unbindKeyCombo(
                    "alt + shift + @Digit" + (i !== 9 ? i + 1 : 0).toString(),
                    handleReadPlayerStats[i],
                )
            }
        }
    }, [selectedKeySlot, game])

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
                        aria-keyshortcuts={t("noShortcut")}
                        aria-label={
                            detect()?.name === "firefox"
                                ? `${t("noShortcut")} ${t("no")}`
                                : ""
                        }
                    >
                        {t("no")}
                    </Button>
                    <Button
                        className="me-2"
                        onClick={async () => await confirm(true)}
                        aria-keyshortcuts={t("yesShortcut")}
                        aria-label={
                            detect()?.name === "firefox"
                                ? `${t("yesShortcut")} ${t("yes")}`
                                : ""
                        }
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
                                onChange={(e) => {
                                    setSelectedKeySlot(e)
                                    setSelectedSlot(e)

                                    let p = game.players.find(
                                        (p) => p.turn_player,
                                    ) as Player

                                    EventManager.publish(Events.slotSelected, {
                                        unavailable: true,
                                        slot: null,
                                        from_year: p.slots[0].to_year,
                                        to_year:
                                            p.slots[p.slots.length - 1]
                                                .from_year,
                                        slot_count: p.slots.length,
                                    } satisfies SlotSelectedData)
                                }}
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
                        onChange={(e) => {
                            let p = game.players.find(
                                (p) => p.turn_player,
                            ) as Player
                            let s = p.slots.find(
                                (s) => s.id === parseInt(e, 10),
                            ) as Slot

                            EventManager.publish(Events.slotSelected, {
                                unavailable: false,
                                slot: s,
                                from_year: p.slots[0].to_year,
                                to_year: p.slots[p.slots.length - 1].from_year,
                                slot_count: p.slots.length,
                            } satisfies SlotSelectedData)
                            setSelectedKeySlot(e)
                            setSelectedSlot(e)
                        }}
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
                        onClick={guess}
                        aria-keyshortcuts={
                            (actionRequired() === PlayerState.Guessing &&
                                selectedSlot !== "0") ||
                            actionRequired() === PlayerState.Intercepting
                                ? t("submitGuessShortcut")
                                : ""
                        }
                        aria-label={
                            detect()?.name === "firefox" &&
                            ((actionRequired() === PlayerState.Guessing &&
                                selectedSlot !== "0") ||
                                actionRequired() === PlayerState.Intercepting)
                                ? `${t("submitGuessShortcut")} ${t("submitGuess")}`
                                : ""
                        }
                    >
                        {actionRequired() === PlayerState.Guessing ||
                        actionRequired() === PlayerState.Intercepting
                            ? actionRequired() === PlayerState.Intercepting ||
                              selectedSlot !== "0"
                                ? t("submitGuess")
                                : t("selectSlotFirst")
                            : t("cannotSubmitGuess")}
                    </Button>
                </>
            )}
        </>
    )
}
