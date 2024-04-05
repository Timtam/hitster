import { createRef, useEffect, useState } from "react"
import Button from "react-bootstrap/Button"
import AudioPlayer from "react-h5-audio-player"
import "react-h5-audio-player/lib/styles.css"

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
                    ? "No hit available"
                    : playing
                      ? "Stop hit"
                      : "Play hit"}
            </Button>
        </>
    )
}
