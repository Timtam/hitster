import EventManager from "@lomray/event-manager"
import { useEffect, useRef, useState } from "react"
import { useTranslation } from "react-i18next"
import {
    Events,
    GuessedData,
    HitRevealedData,
    JoinedGameData,
    LeftGameData,
    TtsData,
} from "./events"

interface SpeechEvent {
    text: string
}

const TIMER_DURATION: number = 150

export default function SpeechPlayer() {
    let { t } = useTranslation()
    let [politeness, setPoliteness] = useState<"polite" | "assertive">("polite")
    let [hidden, setHidden] = useState<boolean>(true)
    let output = useRef<HTMLParagraphElement | null>(null)
    let events = useRef<SpeechEvent[]>([])
    let timer = useRef<ReturnType<typeof setTimeout> | null>(null)

    const handleSpeechEvent = () => {
        if (events.current.length === 0) {
            if (output.current) output.current.innerHTML = ""
            setHidden(true)
            timer.current = null
            return
        }

        if (output.current) output.current.innerHTML = events.current[0].text
        events.current.shift()
        timer.current = setTimeout(handleSpeechEvent, TIMER_DURATION)
    }

    useEffect(() => {
        const unsubscribeTts = EventManager.subscribe(
            Events.tts,
            (e: TtsData) => {
                if (e.interrupt) {
                    setPoliteness("assertive")
                    events.current.length = 0
                } else setPoliteness("polite")
                events.current.push({ text: e.text } satisfies SpeechEvent)
                if (timer.current === null) {
                    setHidden(false)
                    timer.current = setTimeout(
                        handleSpeechEvent,
                        TIMER_DURATION,
                    )
                }
            },
        )

        const unsubscribeJoinedGame = EventManager.subscribe(
            Events.joinedGame,
            (e: JoinedGameData) => {
                EventManager.publish(Events.tts, {
                    text: e.player
                        ? t("otherJoinedGame", { player: e.player.name })
                        : t("youJoinedGame"),
                } satisfies TtsData)
            },
        )

        const unsubscribeLeftGame = EventManager.subscribe(
            Events.leftGame,
            (e: LeftGameData) => {
                EventManager.publish(Events.tts, {
                    text: e.player
                        ? t("otherLeftGame", { player: e.player.name })
                        : t("youLeftGame"),
                } satisfies TtsData)
            },
        )

        const unsubscribeHitRevealed = EventManager.subscribe(
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

        const unsubscribeGuessed = EventManager.subscribe(
            Events.guessed,
            (e: GuessedData) => {
                if (e.player.guess === null)
                    EventManager.publish(Events.tts, {
                        text: t("guessNothing", { player: e.player.name }),
                    } satisfies TtsData)
                else
                    EventManager.publish(Events.tts, {
                        text: t("guess", {
                            player: e.player.name,
                            guess:
                                e.player.guess.from_year === 0
                                    ? t("beforeYear", {
                                          year: e.player.guess.to_year,
                                      })
                                    : e.player.guess.to_year === 0
                                      ? t("afterYear", {
                                            year: e.player.guess.from_year,
                                        })
                                      : t("betweenYears", {
                                            year1: e.player.guess.from_year,
                                            year2: e.player.guess.to_year,
                                        }),
                        }),
                    })
            },
        )

        return () => {
            unsubscribeGuessed()
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
