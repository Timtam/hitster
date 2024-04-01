import { createRef, useEffect, useState } from "react"
import Button from "react-bootstrap/Button"
import AudioPlayer from "react-h5-audio-player"
import "react-h5-audio-player/lib/styles.css"
import { useImmer } from "use-immer"

export default function HitPlayer({
    src,
    duration,
}: {
    src: string
    duration: number
}) {
    let player = createRef<AudioPlayer>()
    let [playing, setPlaying] = useState(false)
    let [timer, setTimer] = useImmer<ReturnType<typeof setTimeout> | undefined>(
        undefined,
    )

    useEffect(() => {
        if (src !== "") {
            setPlaying(true)
            player.current?.audio.current?.play()
        } else {
            setPlaying(false)
            player.current?.audio.current?.pause()
        }
    }, [src])

    useEffect(() => {
        if (playing === true) {
            setTimer(
                setTimeout(() => {
                    setPlaying(false)
                }, duration * 1000),
            )
        } else {
            clearTimeout(timer)
            setTimer(undefined)
            player.current?.audio.current?.load()
        }
    }, [playing])

    return (
        <>
            <AudioPlayer
                ref={player}
                src={src}
                style={{ display: "none" }}
                aria-hidden={true}
                showJumpControls={false}
                showDownloadProgress={false}
                showFilledProgress={false}
                autoPlayAfterSrcChange={false}
            />
            <Button
                disabled={src === ""}
                onClick={() => {
                    if (playing === true) {
                        setPlaying(false)
                        player.current?.audio.current?.load()
                    } else {
                        setPlaying(true)
                        player.current?.audio.current?.play()
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
