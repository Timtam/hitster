import { useLocalStorage } from "@uidotdev/usehooks"
import { Howl } from "howler"
import { useEffect, useState } from "react"
import Button from "react-bootstrap/Button"
import { useTranslation } from "react-i18next"
import playHit from "../../../sfx/play_hit.mp3"
import stopHit from "../../../sfx/stop_hit.mp3"

export default function HitPlayer({
    src,
    duration,
}: {
    src: string
    duration: number
}) {
    let [player, setPlayer] = useState<Howl | undefined>(undefined)
    let [playing, setPlaying] = useState(false)
    let [timer, setTimer] = useState<ReturnType<typeof setTimeout> | undefined>(
        undefined,
    )
    let { t } = useTranslation()
    let [volume] = useLocalStorage("musicVolume", "1.0")
    let [sfxVolume] = useLocalStorage("sfxVolume", "1.0")
    let hPlayHit = new Howl({
        src: [playHit],
    })
    let hStopHit = new Howl({
        src: [stopHit],
    })

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
            hPlayHit.volume(parseFloat(sfxVolume))
            hPlayHit.play()
        } else {
            plr.play()
        }
        if (duration > 0)
            setTimer(
                setTimeout(() => {
                    setPlaying(false)
                }, duration * 1000),
            )
    }

    useEffect(() => {
        if (src !== "" && navigator.userActivation.hasBeenActive) {
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
            if (src !== "" && parseFloat(sfxVolume) > 0) {
                hStopHit.volume(parseFloat(sfxVolume))
                hStopHit.play()
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
}
