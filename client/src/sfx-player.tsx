import EventManager from "@lomray/event-manager"
import { useLocalStorage } from "@uidotdev/usehooks"
import { Howl } from "howler"
import { useEffect, useRef } from "react"
import { GameMode, User } from "./entities"
import {
    ClaimedHitData,
    Events,
    GameEndedData,
    GuessedData,
    PlaySfxData,
    ScoredData,
    Sfx,
    SfxEndedData,
    SlotSelectedData,
    TokenReceivedData,
} from "./events"

const getSfx = (sfx: Sfx): Howl => {
    let url: string

    switch (sfx) {
        case Sfx.noInterception: {
            url = new URL("../sfx/no_interception.opus", import.meta.url).href
            break
        }
        case Sfx.payToken: {
            url = new URL("../sfx/pay_token.opus", import.meta.url).href
            break
        }
        case Sfx.playHit: {
            url = new URL("../sfx/play_hit.opus", import.meta.url).href
            break
        }
        case Sfx.receiveToken: {
            url = new URL("../sfx/receive_token.opus", import.meta.url).href
            break
        }
        case Sfx.selectSlot: {
            url = new URL("../sfx/select_slot.opus", import.meta.url).href
            break
        }
        case Sfx.slotUnavailable: {
            url = new URL("../sfx/slot_unavailable.opus", import.meta.url).href
            break
        }
        case Sfx.stopHit: {
            url = new URL("../sfx/stop_hit.opus", import.meta.url).href
            break
        }
        case Sfx.youFail: {
            url = new URL("../sfx/you_fail.opus", import.meta.url).href
            break
        }
        case Sfx.youLose: {
            url = new URL("../sfx/you_lose.opus", import.meta.url).href
            break
        }
        case Sfx.youScore: {
            url = new URL("../sfx/you_score.opus", import.meta.url).href
            break
        }
        case Sfx.youWin: {
            url = new URL("../sfx/you_win.opus", import.meta.url).href
            break
        }
        case Sfx.joinGame: {
            url = new URL("../sfx/join_game.opus", import.meta.url).href
            break
        }
        case Sfx.leaveGame: {
            url = new URL("../sfx/leave_game.opus", import.meta.url).href
            break
        }
        case Sfx.youClaim: {
            url = new URL("../sfx/claim_hit.opus", import.meta.url).href
            break
        }
        case Sfx.popup: {
            url = new URL("../sfx/popup.opus", import.meta.url).href
            break
        }
    }
    return new Howl({
        src: [url],
        format: "opus",
    })
}

export default function SfxPlayer({ user }: { user: User | null }) {
    const [sfxVolume] = useLocalStorage("sfxVolume", "1.0")
    const sfx = useRef<Map<Sfx, Howl>>(new Map())

    useEffect(() => {
        const unsubscribe = EventManager.subscribe(
            Events.playSfx,
            (e: PlaySfxData) => {
                if (parseFloat(sfxVolume) > 0) {
                    const s = sfx.current.get(e.sfx) ?? getSfx(e.sfx)
                    sfx.current.set(e.sfx, s)

                    s.volume(parseFloat(sfxVolume))
                    s.once("end", () => {
                        EventManager.publish(Events.sfxEnded, {
                            sfx: e.sfx,
                        } satisfies SfxEndedData)
                    })

                    if (e.pan) s.stereo(e.pan)
                    else s.stereo(0)

                    s.play()
                }
            },
        )

        return () => {
            unsubscribe()
        }
    }, [sfxVolume])

    useEffect(() => {
        const unsubscribeGuessed = EventManager.subscribe(
            Events.guessed,
            (e: GuessedData) => {
                if (e.player.guess === null) {
                    EventManager.publish(Events.playSfx, {
                        sfx: Sfx.noInterception,
                    } satisfies PlaySfxData)
                } else {
                    EventManager.publish(Events.playSfx, {
                        sfx: Sfx.payToken,
                    } satisfies PlaySfxData)
                }
            },
        )

        const unsubscribeScored = EventManager.subscribe(
            Events.scored,
            (e: ScoredData) => {
                if (
                    e.winner === user?.id ||
                    (e.winner !== null && e.game_mode === GameMode.Local)
                )
                    EventManager.publish(Events.playSfx, {
                        sfx: Sfx.youScore,
                    } satisfies PlaySfxData)
                else if (
                    e.players.find((p) => p.id === user?.id)?.guess ||
                    (e.game_mode === GameMode.Local && e.winner === null)
                )
                    EventManager.publish(Events.playSfx, {
                        sfx: Sfx.youFail,
                    } satisfies PlaySfxData)
            },
        )

        const unsubscribeReceivedToken = EventManager.subscribe(
            Events.tokenReceived,
            (e: TokenReceivedData) => {
                if (e.player.id === user?.id || e.game_mode === GameMode.Local)
                    EventManager.publish(Events.playSfx, {
                        sfx: Sfx.receiveToken,
                    } satisfies PlaySfxData)
            },
        )

        const unsubscribeClaimed = EventManager.subscribe(
            Events.claimedHit,
            (e: ClaimedHitData) => {
                if (e.player.id === user?.id || e.game_mode === GameMode.Local)
                    EventManager.publish(Events.playSfx, {
                        sfx: Sfx.youClaim,
                    } satisfies PlaySfxData)
            },
        )

        const unsubscribeGameEnded = EventManager.subscribe(
            Events.gameEnded,
            (e: GameEndedData) => {
                if (
                    e.game.mode !== GameMode.Local &&
                    e.winner?.id !== user?.id &&
                    e.game.players.find((p) => p.id === user?.id) !== undefined
                )
                    EventManager.publish(Events.playSfx, {
                        sfx: Sfx.youLose,
                    } satisfies PlaySfxData)
                else if (
                    e.winner?.id === user?.id ||
                    (e.game.mode === GameMode.Local && e.winner !== null)
                )
                    EventManager.publish(Events.playSfx, {
                        sfx: Sfx.youWin,
                    } satisfies PlaySfxData)
            },
        )

        const unsubscribeJoinedGame = EventManager.subscribe(
            Events.joinedGame,
            () => {
                EventManager.publish(Events.playSfx, {
                    sfx: Sfx.joinGame,
                } satisfies PlaySfxData)
            },
        )

        const unsubscribeLeftGame = EventManager.subscribe(
            Events.leftGame,
            () => {
                EventManager.publish(Events.playSfx, {
                    sfx: Sfx.leaveGame,
                } satisfies PlaySfxData)
            },
        )

        const unsubscribeSlotSelected = EventManager.subscribe(
            Events.slotSelected,
            (e: SlotSelectedData) => {
                let pan = 0

                if (e.slot) {
                    if (e.slot.from_year === 0) pan = -1
                    else if (e.slot.to_year === 0) pan = 1
                    else
                        pan =
                            -1 +
                            2 *
                                ((e.slot.from_year +
                                    (e.slot.to_year - e.slot.from_year) / 2 -
                                    e.from_year) /
                                    (e.to_year - e.from_year))

                    EventManager.publish(Events.playSfx, {
                        sfx: Sfx.selectSlot,
                        pan: pan,
                    } satisfies PlaySfxData)
                }

                if (e.unavailable || e.slot === null)
                    EventManager.publish(Events.playSfx, {
                        sfx: Sfx.slotUnavailable,
                        pan: pan,
                    } satisfies PlaySfxData)
            },
        )

        const unsubscribePopup = EventManager.subscribe(Events.popup, () => {
            EventManager.publish(Events.playSfx, {
                sfx: Sfx.popup,
            } satisfies PlaySfxData)
        })

        return () => {
            unsubscribeClaimed()
            unsubscribeGuessed()
            unsubscribeScored()
            unsubscribeGameEnded()
            unsubscribeJoinedGame()
            unsubscribeLeftGame()
            unsubscribeReceivedToken()
            unsubscribeSlotSelected()
            unsubscribePopup()
        }
    }, [user])

    return <> </>
}
