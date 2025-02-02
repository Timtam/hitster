import EventManager from "@lomray/event-manager"
import {
    bindKeyCombo,
    BrowserKeyComboEvent,
    unbindKeyCombo,
} from "@rwh/keystrokes"
import { useLocalStorage } from "@uidotdev/usehooks"
import { detect } from "detect-browser"
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
import { useEffectEvent } from "use-effect-event"
import { Events, Sfx } from "../../events"
import { useModalShown } from "../../hooks"

interface HitPlayerTimers {
    sfxTimer: ReturnType<typeof setTimeout> | null
    stopTimer: ReturnType<typeof setTimeout> | null
}

export type HitPlayerProps = {
    src: string
    duration: number
    onPlay?: () => void
    autoplay?: boolean
    shortcut?: string
}

export type HitPlayerRef = {
    play: () => void
    stop: () => void
}

export const HitPlayer = forwardRef<HitPlayerRef, HitPlayerProps>(
    function HitPlayer(
        { src, duration, onPlay, autoplay, shortcut }: HitPlayerProps,
        ref,
    ) {
        const player = useRef<Howl | null>(null)
        const [playing, setPlaying] = useState(false)
        const timers = useRef<HitPlayerTimers>({
            sfxTimer: null,
            stopTimer: null,
        } satisfies HitPlayerTimers)
        const { t } = useTranslation()
        const [volume] = useLocalStorage("musicVolume", "1.0")
        const [sfxVolume] = useLocalStorage("sfxVolume", "1.0")
        const modalShown = useModalShown()

        const play = useEffectEvent(() => {
            if (timers.current.stopTimer) {
                clearTimeout(timers.current.stopTimer)
            }
            player.current?.stop()
            const plr = new Howl({
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
        })

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
        }, [autoplay, setPlaying, src])

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
        }, [play, sfxVolume, src, playing])

        useEffect(() => {
            const handlePlayOrStopHit = {
                onPressed: (e: BrowserKeyComboEvent) => {
                    e.finalKeyEvent.preventDefault()
                    if (playing) setPlaying(false)
                    else setPlaying(true)
                },
            }

            if (shortcut !== undefined && src !== "" && !modalShown) {
                bindKeyCombo("alt + shift + h", handlePlayOrStopHit)
            }

            return () => {
                unbindKeyCombo("alt + shift + h", handlePlayOrStopHit)
            }
        }, [playing, src, shortcut, modalShown])

        useEffect(() => {
            player.current?.volume(parseFloat(volume))
        }, [volume])

        /* eslint-disable react-hooks/exhaustive-deps */
        // eslint warns about timers.current most likely being reset already
        // we however are using timers not for linking a node, but for storing timer ids

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

        /* eslint-enable react-hooks/exhaustive-deps */

        return (
            <>
                <Button
                    className="me-2"
                    disabled={src === ""}
                    aria-keyshortcuts={shortcut !== undefined ? shortcut : ""}
                    aria-label={
                        detect()?.name === "firefox" &&
                        shortcut !== undefined &&
                        src !== ""
                            ? `${shortcut} ${playing ? t("stopHit") : t("playHit")}`
                            : ""
                    }
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
