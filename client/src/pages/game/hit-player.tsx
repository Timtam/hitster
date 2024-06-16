import EventManager from "@lomray/event-manager"
import { useLocalStorage } from "@uidotdev/usehooks"
import { Howl } from "howler"
import { forwardRef, useEffect, useImperativeHandle, useState } from "react"
import Button from "react-bootstrap/Button"
import { useTranslation } from "react-i18next"
import { Events, Sfx } from "../../events"

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
        let [player, setPlayer] = useState<Howl | undefined>(undefined)
        let [playing, setPlaying] = useState(false)
        let [timer, setTimer] = useState<
            ReturnType<typeof setTimeout> | undefined
        >(undefined)
        let { t } = useTranslation()
        let [volume] = useLocalStorage("musicVolume", "1.0")
        let [sfxVolume] = useLocalStorage("sfxVolume", "1.0")

        const play = () => {
            if (timer !== undefined) {
                clearTimeout(timer)
            }
            if (player !== undefined) player.stop()
            let plr = new Howl({
                src: [src],
                format: "audio/mpeg",
                html5: true,
                volume: parseFloat(volume),
            })
            plr.once("end", () => {
                setPlaying(false)
            })
            setPlayer(plr)
            if (parseFloat(sfxVolume) > 0) {
                setTimeout(() => plr.play(), 250)
                EventManager.publish(Events.playSfx, { sfx: Sfx.playHit })
            } else {
                plr.play()
            }
            if (duration > 0)
                setTimer(
                    setTimeout(() => {
                        setPlaying(false)
                    }, duration * 1000),
                )
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
                if (timer !== undefined) {
                    clearTimeout(timer)
                    setTimer(undefined)
                }
                player?.pause()
                if (
                    src !== "" &&
                    parseFloat(sfxVolume) > 0 &&
                    player !== undefined
                ) {
                    EventManager.publish(Events.playSfx, { sfx: Sfx.stopHit })
                }
                setPlayer(undefined)
            }
        }, [src, playing])

        useEffect(() => {
            if (player !== undefined) player.volume(parseFloat(volume))
        }, [volume])

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
