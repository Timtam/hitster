import EventManager from "@lomray/event-manager"
import { useLocalStorage } from "@uidotdev/usehooks"
import { Howl } from "howler"
import {
    forwardRef,
    useEffect,
    useImperativeHandle,
    useRef,
    useState,
} from "react"
import Button from "react-bootstrap/Button"
import { useTranslation } from "react-i18next"
import { Events, Sfx } from "../../events"

interface HitPlayerTimers {
    sfxTimer: ReturnType<typeof setTimeout> | null
    stopTimer: ReturnType<typeof setTimeout> | null
}

export type HitPlayerProps = {
    src: string
    duration: number
    onPlay?: () => void
    autoplay?: boolean
}

export type HitPlayerRef = {
    play: () => void
    stop: () => void
}

export const HitPlayer = forwardRef<HitPlayerRef, HitPlayerProps>(
    function HitPlayer(
        { src, duration, onPlay, autoplay }: HitPlayerProps,
        ref,
    ) {
        let player = useRef<Howl | null>(null)
        let [playing, setPlaying] = useState(false)
        let timers = useRef<HitPlayerTimers>({
            sfxTimer: null,
            stopTimer: null,
        } satisfies HitPlayerTimers)
        let { t } = useTranslation()
        let [volume] = useLocalStorage("musicVolume", "1.0")
        let [sfxVolume] = useLocalStorage("sfxVolume", "1.0")

        const play = () => {
            if (timers.current.stopTimer) {
                clearTimeout(timers.current.stopTimer)
            }
            player.current?.stop()
            let plr = new Howl({
                src: [src],
                format: "audio/mpeg",
                html5: true,
                volume: parseFloat(volume),
            })
            plr.once("end", () => {
                setPlaying(false)
            })
            player.current = plr
            if (parseFloat(sfxVolume) > 0) {
                timers.current.sfxTimer = setTimeout(() => {
                    plr.play()
                    timers.current.sfxTimer = null
                }, 250)
                EventManager.publish(Events.playSfx, { sfx: Sfx.playHit })
            } else {
                plr.play()
            }
            if (duration > 0)
                timers.current.stopTimer = setTimeout(() => {
                    setPlaying(false)
                }, duration * 1000)
            if (onPlay !== undefined) onPlay()
        }

        const stop = () => {
            setPlaying(false)
        }

        useImperativeHandle(
            ref,
            () =>
                ({
                    play: play,
                    stop: stop,
                }) satisfies HitPlayerRef,
        )

        useEffect(() => {
            if (
                src !== "" &&
                navigator.userActivation.hasBeenActive &&
                autoplay !== false
            ) {
                setPlaying(true)
            } else {
                setPlaying(false)
            }
        }, [src])

        useEffect(() => {
            if (playing === true) {
                play()
            } else {
                if (timers.current.stopTimer) {
                    clearTimeout(timers.current.stopTimer)
                    timers.current.stopTimer = null
                }
                if (timers.current.sfxTimer) {
                    clearTimeout(timers.current.sfxTimer)
                    timers.current.sfxTimer = null
                }
                player.current?.pause()
                if (
                    src !== "" &&
                    parseFloat(sfxVolume) > 0 &&
                    player.current !== null
                ) {
                    EventManager.publish(Events.playSfx, { sfx: Sfx.stopHit })
                }
                player.current = null
            }
        }, [src, playing])

        useEffect(() => {
            player.current?.volume(parseFloat(volume))
        }, [volume])

        useEffect(() => {
            return () => {
                if (timers.current.stopTimer) {
                    clearTimeout(timers.current.stopTimer)
                    timers.current.stopTimer = null
                }
                if (timers.current.sfxTimer) {
                    clearTimeout(timers.current.sfxTimer)
                    timers.current.sfxTimer = null
                }
                player.current?.pause()
                player.current = null
            }
        }, [])

        return (
            <>
                <Button
                    className="me-2"
                    disabled={src === ""}
                    onClick={() => {
                        if (playing === true) {
                            setPlaying(false)
                        } else {
                            setPlaying(true)
                        }
                    }}
                >
                    {src === ""
                        ? t("noHitAvailable")
                        : playing
                          ? t("stopHit")
                          : t("playHit")}
                </Button>
            </>
        )
    },
)
