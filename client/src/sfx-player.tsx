import EventManager from "@lomray/event-manager"
import { useLocalStorage } from "@uidotdev/usehooks"
import { Howl } from "howler"
import { useEffect } from "react"
import { Events, PlaySfxData, Sfx } from "./events"

// sfx imports
import noInterception from "../sfx/no_interception.mp3"
import payToken from "../sfx/pay_token.mp3"
import playHit from "../sfx/play_hit.mp3"
import stopHit from "../sfx/stop_hit.mp3"
import youFail from "../sfx/you_fail.mp3"
import youLose from "../sfx/you_lose.mp3"
import youScore from "../sfx/you_score.mp3"
import youWin from "../sfx/you_win.mp3"

const getSfx = (sfx: Sfx): Howl => {
    let url: string

    switch (sfx) {
        case Sfx.noInterception: {
            url = noInterception
            break
        }
        case Sfx.payToken: {
            url = payToken
            break
        }
        case Sfx.playHit: {
            url = playHit
            break
        }
        case Sfx.stopHit: {
            url = stopHit
            break
        }
        case Sfx.youFail: {
            url = youFail
            break
        }
        case Sfx.youLose: {
            url = youLose
            break
        }
        case Sfx.youScore: {
            url = youScore
            break
        }
        case Sfx.youWin: {
            url = youWin
            break
        }
    }
    return new Howl({
        src: [url],
        format: "audio/mpeg",
        html5: true,
    })
}

export default function SfxPlayer() {
    let [sfxVolume] = useLocalStorage("sfxVolume", "1.0")
    let sfx: Map<Sfx, Howl> = new Map()

    useEffect(() => {
        const playSfx = (e: PlaySfxData) => {
            if (parseFloat(sfxVolume) > 0) {
                let s = sfx.get(e.sfx) ?? getSfx(e.sfx)
                sfx.set(e.sfx, s)

                s.volume(parseFloat(sfxVolume))
                s.once("end", () => {
                    EventManager.publish(Events.sfxEnded, {
                        sfx: e.sfx,
                    })
                })

                s.play()
            }
        }

        let unsubscribe = EventManager.subscribe(Events.playSfx, playSfx)

        return () => {
            unsubscribe()
        }
    }, [sfxVolume])

    return <> </>
}
