import EventManager from "@lomray/event-manager"
import {
    bindKeyCombo,
    BrowserKeyComboEvent,
    unbindKeyCombo,
} from "@rwh/keystrokes"
import { detect } from "detect-browser"
import { useCallback, useEffect, useState } from "react"
import Button from "react-bootstrap/Button"
import OverlayTrigger from "react-bootstrap/OverlayTrigger"
import Tooltip from "react-bootstrap/Tooltip"
import { Trans, useTranslation } from "react-i18next"
import { useContext } from "../../context"
import type { Game, Slot } from "../../entities"
import { GameMode, GameState, Player, PlayerState } from "../../entities"
import { Events, NotificationData, SlotSelectedData } from "../../events"
import GameService from "../../services/games.service"

export default function SlotSelector({ game }: { game: Game }) {
    const { user } = useContext()
    const [selectedSlot, setSelectedSlot] = useState("0")
    const [selectedKeySlot, setSelectedKeySlot] = useState("0")
    const { t } = useTranslation()

    const actionPlayer = useCallback((): Player | null => {
        if (game.state === GameState.Open) return null

        const me = game.players.find((p) => p.id === user?.id)

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
    }, [game, user])

    const actionRequired = useCallback((): PlayerState => {
        if (user === null) return PlayerState.Waiting
        return actionPlayer()?.state ?? PlayerState.Waiting
    }, [actionPlayer, user])

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

    const confirm = useCallback(
        async (confirm: boolean) => {
            try {
                const gs = new GameService()
                await gs.confirm(game.id, confirm)
            } catch (e) {
                console.log(e)
            }
        },
        [game],
    )

    const guess = useCallback(async () => {
        let slot: number | null =
            selectedSlot !== "0" ? parseInt(selectedSlot, 10) : null

        if (game.players.some((p) => p.guess?.id === slot)) slot = null

        if (actionRequired() !== PlayerState.Intercepting && slot === null)
            return

        try {
            const gs = new GameService()
            await gs.guess(
                game.id,
                slot,
                game.mode === GameMode.Local ? actionPlayer()?.id : undefined,
            )
            setSelectedSlot("0")
            setSelectedKeySlot("0")
        } catch (e) {
            console.log(e)
        }
    }, [actionPlayer, actionRequired, game, selectedSlot])

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
        const handlePreviousSlot = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()
                let slot = "0"
                const p = game.players.find((p) => p.turn_player) as Player

                if (selectedKeySlot === "0" || selectedKeySlot === "1")
                    slot = p.slots.length.toString()
                else slot = (parseInt(selectedKeySlot, 10) - 1).toString()

                const s = p.slots.find(
                    (s) => s.id === parseInt(slot, 10),
                ) as Slot
                const u =
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

        const handleNextSlot = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()

                let slot = "0"
                const p = game.players.find((p) => p.turn_player) as Player

                if (
                    selectedKeySlot === "0" ||
                    selectedKeySlot === p.slots.length.toString()
                )
                    slot = "1"
                else slot = (parseInt(selectedKeySlot, 10) + 1).toString()

                const s = p.slots.find(
                    (s) => s.id === parseInt(slot, 10),
                ) as Slot
                const u =
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

        const handleResetSlot = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()
                if (selectedKeySlot !== "0") {
                    setSelectedKeySlot("0")
                    setSelectedSlot("0")

                    const p = game.players.find((p) => p.turn_player) as Player

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

        const handleGuess = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()
                guess()
            },
        }

        const handleConfirmYes = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()
                confirm(true)
            },
        }

        const handleConfirmNo = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()
                confirm(false)
            },
        }

        const handleReadPlayerStats = Array.from({ length: 10 }, (_, i) => ({
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
    }, [actionRequired, confirm, game, guess, selectedKeySlot, t])

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
                        <div className="form-check mb-2">
                            <input
                                className="form-check-input"
                                id="slot-0"
                                key="slot-0"
                                value="0"
                                type="radio"
                                checked={selectedSlot === "0"}
                                onChange={(e) => {
                                    setSelectedKeySlot(e.target.value)
                                    setSelectedSlot(e.target.value)

                                    const p = game.players.find(
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
                            />
                            <label
                                htmlFor="slot-0"
                                className="form-check-label"
                            >
                                {t("dontIntercept")}
                            </label>
                        </div>
                    ) : (
                        ""
                    )}
                    <div className="row mb-2">
                        {game.players
                            .find((p) => p.turn_player === true)
                            ?.slots.map((slot) => {
                                const disabled =
                                    (actionRequired() !==
                                        PlayerState.Guessing &&
                                        actionRequired() !==
                                            PlayerState.Intercepting) ||
                                    game.players.some(
                                        (p) => p.guess?.id === slot.id,
                                    )

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
                                        <OverlayTrigger
                                            overlay={(props) =>
                                                disabled ? (
                                                    <Tooltip
                                                        id={`slot-tooltip-${slot.id}`}
                                                        {...props}
                                                        className="opacity-100"
                                                    >
                                                        {" "}
                                                        <div>
                                                            {game.players.find(
                                                                (p) =>
                                                                    p.guess
                                                                        ?.id ===
                                                                    slot.id,
                                                            )?.name ?? <div />}
                                                        </div>
                                                    </Tooltip>
                                                ) : (
                                                    <div />
                                                )
                                            }
                                        >
                                            <div className="col-auto mb-2">
                                                <input
                                                    className="form-check-input mb-2"
                                                    value={slot.id.toString()}
                                                    key={`slot-${slot.id.toString()}`}
                                                    id={`slot-${slot.id.toString()}`}
                                                    type="radio"
                                                    disabled={disabled}
                                                    checked={
                                                        selectedSlot ===
                                                            slot.id.toString() ||
                                                        game.players.some(
                                                            (p) =>
                                                                p.guess?.id ===
                                                                slot.id,
                                                        )
                                                    }
                                                    onChange={(e) => {
                                                        const p =
                                                            game.players.find(
                                                                (p) =>
                                                                    p.turn_player,
                                                            ) as Player
                                                        const s = p.slots.find(
                                                            (s) =>
                                                                s.id ===
                                                                parseInt(
                                                                    e.target
                                                                        .value,
                                                                    10,
                                                                ),
                                                        ) as Slot

                                                        EventManager.publish(
                                                            Events.slotSelected,
                                                            {
                                                                unavailable:
                                                                    false,
                                                                slot: s,
                                                                from_year:
                                                                    p.slots[0]
                                                                        .to_year,
                                                                to_year:
                                                                    p.slots[
                                                                        p.slots
                                                                            .length -
                                                                            1
                                                                    ].from_year,
                                                                slot_count:
                                                                    p.slots
                                                                        .length,
                                                            } satisfies SlotSelectedData,
                                                        )
                                                        setSelectedKeySlot(
                                                            e.target.value,
                                                        )
                                                        setSelectedSlot(
                                                            e.target.value,
                                                        )
                                                    }}
                                                    aria-label={
                                                        text +
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
                                                            : "")
                                                    }
                                                    style={
                                                        disabled
                                                            ? {
                                                                  pointerEvents:
                                                                      "none",
                                                              }
                                                            : {}
                                                    }
                                                />
                                            </div>
                                        </OverlayTrigger>
                                        {slot.to_year !== 0 ? (
                                            <div
                                                className="col-auto g-0 mb-2"
                                                aria-hidden={true}
                                            >
                                                {slot.to_year}
                                            </div>
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
