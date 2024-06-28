import EventManager from "@lomray/event-manager"
import { useEffect, useRef, useState } from "react"
import { useTranslation } from "react-i18next"
import {
    Events,
    HitRevealedData,
    JoinedGameData,
    LeftGameData,
    TtsData,
} from "./events"

export default function SpeechPlayer() {
    let { t } = useTranslation()
    let [politeness, setPoliteness] = useState<"polite" | "assertive">("polite")
    let [hidden, setHidden] = useState<boolean>(true)
    let output = useRef<HTMLParagraphElement | null>(null)

    useEffect(() => {
        let unsubscribeTts = EventManager.subscribe(
            Events.tts,
            (e: TtsData) => {
                if (e.interrupt) setPoliteness("assertive")
                else setPoliteness("polite")
                setHidden(false)
                setTimeout(() => {
                    if (output.current) output.current.innerHTML = e.text
                    setTimeout(() => {
                        setHidden(true)
                    }, 2000)
                }, 150)
            },
        )

        let unsubscribeJoinedGame = EventManager.subscribe(
            Events.joinedGame,
            (e: JoinedGameData) => {
                EventManager.publish(Events.tts, {
                    text: e.player
                        ? t("otherJoinedGame", { player: e.player.name })
                        : t("youJoinedGame"),
                } satisfies TtsData)
            },
        )

        let unsubscribeLeftGame = EventManager.subscribe(
            Events.leftGame,
            (e: LeftGameData) => {
                EventManager.publish(Events.tts, {
                    text: e.player
                        ? t("otherLeftGame", { player: e.player.name })
                        : t("youLeftGame"),
                } satisfies TtsData)
            },
        )

        let unsubscribeHitRevealed = EventManager.subscribe(
            Events.hitRevealed,
            (e: HitRevealedData) => {
                EventManager.publish(Events.tts, {
                    text:
                        e.hit.belongs_to !== ""
                            ? t("hitRevealedBelongingShort", {
                                  title: e.hit.title,
                                  year: e.hit.year,
                                  artist: e.hit.artist,
                                  belongs_to: e.hit.belongs_to,
                                  pack: e.hit.pack,
                                  player: e.player?.name ?? t("noone"),
                              })
                            : t("hitRevealedShort", {
                                  title: e.hit.title,
                                  artist: e.hit.artist,
                                  year: e.hit.year,
                                  pack: e.hit.pack,
                                  player: e.player?.name ?? t("noone"),
                              }),
                } satisfies TtsData)
            },
        )

        return () => {
            unsubscribeHitRevealed()
            unsubscribeJoinedGame()
            unsubscribeLeftGame()
            unsubscribeTts()
        }
    }, [])

    return (
        <p
            aria-live={politeness}
            aria-atomic={true}
            ref={output}
            className="visually-hidden"
            aria-hidden={hidden}
        />
    )
}
