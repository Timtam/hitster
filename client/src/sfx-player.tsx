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
    }
    return new Howl({
        src: [url],
        format: "opus",
    })
}

export default function SfxPlayer({ user }: { user: User | null }) {
    let [sfxVolume] = useLocalStorage("sfxVolume", "1.0")
    let sfx = useRef<Map<Sfx, Howl>>(new Map())

    useEffect(() => {
        let unsubscribe = EventManager.subscribe(
            Events.playSfx,
            (e: PlaySfxData) => {
                if (parseFloat(sfxVolume) > 0) {
                    let s = sfx.current.get(e.sfx) ?? getSfx(e.sfx)
                    sfx.current.set(e.sfx, s)

                    s.volume(parseFloat(sfxVolume))
                    s.once("end", () => {
                        EventManager.publish(Events.sfxEnded, {
                            sfx: e.sfx,
                        } satisfies SfxEndedData)
                    })

                    s.play()
                }
            },
        )

        return () => {
            unsubscribe()
        }
    }, [sfxVolume])

    useEffect(() => {
        let unsubscribeGuessed = EventManager.subscribe(
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

        let unsubscribeScored = EventManager.subscribe(
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

        let unsubscribeReceivedToken = EventManager.subscribe(
            Events.tokenReceived,
            (e: TokenReceivedData) => {
                if (e.player.id === user?.id || e.game_mode === GameMode.Local)
                    EventManager.publish(Events.playSfx, {
                        sfx: Sfx.receiveToken,
                    } satisfies PlaySfxData)
            },
        )

        let unsubscribeClaimed = EventManager.subscribe(
            Events.claimedHit,
            (e: ClaimedHitData) => {
                if (e.player.id === user?.id || e.game_mode === GameMode.Local)
                    EventManager.publish(Events.playSfx, {
                        sfx: Sfx.youClaim,
                    } satisfies PlaySfxData)
            },
        )

        let unsubscribeGameEnded = EventManager.subscribe(
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

        let unsubscribeJoinedGame = EventManager.subscribe(
            Events.joinedGame,
            () => {
                EventManager.publish(Events.playSfx, {
                    sfx: Sfx.joinGame,
                } satisfies PlaySfxData)
            },
        )

        let unsubscribeLeftGame = EventManager.subscribe(
            Events.leftGame,
            () => {
                EventManager.publish(Events.playSfx, {
                    sfx: Sfx.leaveGame,
                } satisfies PlaySfxData)
            },
        )

        return () => {
            unsubscribeClaimed()
            unsubscribeGuessed()
            unsubscribeScored()
            unsubscribeGameEnded()
            unsubscribeJoinedGame()
            unsubscribeLeftGame()
            unsubscribeReceivedToken()
        }
    }, [user])

    return <> </>
}
