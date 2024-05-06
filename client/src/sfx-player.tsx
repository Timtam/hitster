import EventManager from "@lomray/event-manager"
import { useLocalStorage } from "@uidotdev/usehooks"
import { Howl } from "howler"
import { useEffect } from "react"
import { GameMode, User } from "./entities"
import {
    Events,
    GameEndedData,
    GuessedData,
    PlaySfxData,
    ScoredData,
    Sfx,
    SfxEndedData,
} from "./events"

const getSfx = (sfx: Sfx): Howl => {
    let url: string

    switch (sfx) {
        case Sfx.noInterception: {
            url = new URL("../sfx/no_interception.mp3", import.meta.url).href
            break
        }
        case Sfx.payToken: {
            url = new URL("../sfx/pay_token.mp3", import.meta.url).href
            break
        }
        case Sfx.playHit: {
            url = new URL("../sfx/play_hit.mp3", import.meta.url).href
            break
        }
        case Sfx.stopHit: {
            url = new URL("../sfx/stop_hit.mp3", import.meta.url).href
            break
        }
        case Sfx.youFail: {
            url = new URL("../sfx/you_fail.mp3", import.meta.url).href
            break
        }
        case Sfx.youLose: {
            url = new URL("../sfx/you_lose.mp3", import.meta.url).href
            break
        }
        case Sfx.youScore: {
            url = new URL("../sfx/you_score.mp3", import.meta.url).href
            break
        }
        case Sfx.youWin: {
            url = new URL("../sfx/you_win.mp3", import.meta.url).href
            break
        }
    }
    return new Howl({
        src: [url],
        format: "audio/mpeg",
        html5: true,
    })
}

export default function SfxPlayer({ user }: { user: User | null }) {
    let [sfxVolume] = useLocalStorage("sfxVolume", "1.0")
    let sfx: Map<Sfx, Howl> = new Map()

    useEffect(() => {
        let unsubscribe = EventManager.subscribe(
            Events.playSfx,
            (e: PlaySfxData) => {
                if (parseFloat(sfxVolume) > 0) {
                    let s = sfx.get(e.sfx) ?? getSfx(e.sfx)
                    sfx.set(e.sfx, s)

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
                if (e.guess === null) {
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
                if (e.winner === user?.id)
                    EventManager.publish(Events.playSfx, {
                        sfx: Sfx.youScore,
                    } satisfies PlaySfxData)
                else if (
                    e.players.find((p) => p.id === user?.id)?.guess !== null
                )
                    EventManager.publish(Events.playSfx, {
                        sfx: Sfx.youFail,
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

        return () => {
            unsubscribeGuessed()
            unsubscribeScored()
            unsubscribeGameEnded()
        }
    }, [user])

    return <> </>
}
