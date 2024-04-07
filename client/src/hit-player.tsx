import { useLocalStorage } from "@uidotdev/usehooks"
import { createRef, useEffect, useState } from "react"
import Button from "react-bootstrap/Button"
import AudioPlayer from "react-h5-audio-player"
import "react-h5-audio-player/lib/styles.css"
import { useTranslation } from "react-i18next"

export default function HitPlayer({
    src,
    duration,
}: {
    src: string
    duration: number
}) {
    let player = createRef<AudioPlayer>()
    let [playing, setPlaying] = useState(false)
    let [timer, setTimer] = useState<ReturnType<typeof setTimeout> | undefined>(
        undefined,
    )
    let { t } = useTranslation()
    let [volume] = useLocalStorage("musicVolume", "1.0")

    useEffect(() => {
        if (src !== "") {
            setPlaying(true)
        } else {
            setPlaying(false)
        }
    }, [src])

    useEffect(() => {
        if (playing === true) {
            if (timer !== undefined) {
                clearTimeout(timer)
            }
            player.current?.audio.current?.play()
            if (duration > 0)
                setTimer(
                    setTimeout(() => {
                        setPlaying(false)
                    }, duration * 1000),
                )
        } else {
            if (timer !== undefined) {
                clearTimeout(timer)
                setTimer(undefined)
            }
            player.current?.audio.current?.pause()
            if (
                player.current !== null &&
                player.current.audio.current !== null
            )
                player.current.audio.current.currentTime = 0
        }
    }, [src, playing])

    useEffect(() => {
        if (player.current !== null && player.current.audio.current !== null)
            player.current.audio.current.volume = parseFloat(volume)
    }, [volume])

    return (
        <>
            <AudioPlayer
                ref={player}
                src={src}
                style={{ display: "none" }}
                showJumpControls={false}
                showDownloadProgress={false}
                showFilledProgress={false}
                autoPlayAfterSrcChange={false}
                volume={parseFloat(volume)}
                onEnded={() => setPlaying(false)}
            />
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
