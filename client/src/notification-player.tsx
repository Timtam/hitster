import EventManager from "@lomray/event-manager"
import { ReactNode, useEffect, useRef, useState } from "react"
import { useToasts } from "react-bootstrap-toasts"
import { flushSync } from "react-dom"
import { createRoot } from "react-dom/client"
import { Trans, useTranslation } from "react-i18next"
import { User } from "./entities"
import {
    Events,
    GuessedData,
    HitRevealedData,
    JoinedGameData,
    LeftGameData,
    NotificationData,
    SkippedHitData,
} from "./events"

interface SpeechEvent {
    text: string
}

const TIMER_DURATION: number = 150

export default function NotificationPlayer({ user }: { user: User | null }) {
    let { t } = useTranslation()
    let [politeness, setPoliteness] = useState<"polite" | "assertive">("polite")
    let [hidden, setHidden] = useState<boolean>(true)
    let output = useRef<HTMLParagraphElement | null>(null)
    let events = useRef<SpeechEvent[]>([])
    let timer = useRef<ReturnType<typeof setTimeout> | null>(null)
    const toasts = useToasts()

    const nodeToString = (node: ReactNode) => {
        const div = document.createElement("div")
        const root = createRoot(div)
        flushSync(() => root.render(node))
        return div.innerText // or innerHTML or textContent
    }

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
        const unsubscribeNotification = EventManager.subscribe(
            Events.notification,
            (e: NotificationData) => {
                if (e.interruptTts) {
                    setPoliteness("assertive")
                    events.current.length = 0
                } else setPoliteness("polite")
                events.current.push({
                    text:
                        typeof e.text === "string"
                            ? e.text
                            : nodeToString(e.text),
                } satisfies SpeechEvent)
                toasts.show({
                    headerContent: "",
                    bodyContent: e.text,
                    toastProps: {
                        autohide: true,
                        delay: 5000,
                    },
                })
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
                EventManager.publish(Events.notification, {
                    text: e.player
                        ? t("otherJoinedGame", { player: e.player.name })
                        : t("youJoinedGame"),
                } satisfies NotificationData)
            },
        )

        const unsubscribeLeftGame = EventManager.subscribe(
            Events.leftGame,
            (e: LeftGameData) => {
                EventManager.publish(Events.notification, {
                    text:
                        e.player.id !== user?.id
                            ? t("otherLeftGame", { player: e.player.name })
                            : t("youLeftGame"),
                } satisfies NotificationData)
            },
        )

        const unsubscribeHitRevealed = EventManager.subscribe(
            Events.hitRevealed,
            (e: HitRevealedData) => {
                EventManager.publish(Events.notification, {
                    text:
                        e.hit.belongs_to !== "" ? (
                            <Trans
                                i18nKey="hitRevealedBelonging"
                                values={{
                                    title: e.hit.title,
                                    artist: e.hit.artist,
                                    year: e.hit.year,
                                    pack: e.hit.pack,
                                    belongs_to: e.hit.belongs_to,
                                    player: e.player?.name ?? t("noone"),
                                }}
                                components={[
                                    <b />,
                                    <b />,
                                    <b />,
                                    <b />,
                                    <b />,
                                    <b />,
                                ]}
                            />
                        ) : (
                            <Trans
                                i18nKey="hitRevealed"
                                values={{
                                    title: e.hit.title,
                                    artist: e.hit.artist,
                                    year: e.hit.year,
                                    pack: e.hit.pack,
                                    player: e.player?.name ?? t("noone"),
                                }}
                                components={[<b />, <b />, <b />, <b />, <b />]}
                            />
                        ),
                } satisfies NotificationData)
            },
        )

        const unsubscribeGuessed = EventManager.subscribe(
            Events.guessed,
            (e: GuessedData) => {
                if (e.player.guess === null)
                    EventManager.publish(Events.notification, {
                        text: t("guessNothing", { player: e.player.name }),
                    } satisfies NotificationData)
                else
                    EventManager.publish(Events.notification, {
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
                    } satisfies NotificationData)
            },
        )

        const unsubscribeSkippedHit = EventManager.subscribe(
            Events.skippedHit,
            (e: SkippedHitData) => {
                EventManager.publish(Events.notification, {
                    text:
                        e.player.id !== user?.id ? (
                            e.hit.belongs_to !== "" ? (
                                <Trans
                                    i18nKey="otherSkippedHitBelonging"
                                    values={{
                                        title: e.hit.title,
                                        artist: e.hit.artist,
                                        year: e.hit.year,
                                        pack: e.hit.pack,
                                        belongs_to: e.hit.belongs_to,
                                        player: e.player?.name ?? t("noone"),
                                    }}
                                    components={[
                                        <b />,
                                        <b />,
                                        <b />,
                                        <b />,
                                        <b />,
                                        <b />,
                                    ]}
                                />
                            ) : (
                                <Trans
                                    i18nKey="otherSkippedHit"
                                    values={{
                                        title: e.hit.title,
                                        artist: e.hit.artist,
                                        year: e.hit.year,
                                        pack: e.hit.pack,
                                        player: e.player?.name ?? t("noone"),
                                    }}
                                    components={[
                                        <b />,
                                        <b />,
                                        <b />,
                                        <b />,
                                        <b />,
                                    ]}
                                />
                            )
                        ) : e.hit.belongs_to !== "" ? (
                            <Trans
                                i18nKey="youSkippedHitBelonging"
                                values={{
                                    title: e.hit.title,
                                    artist: e.hit.artist,
                                    year: e.hit.year,
                                    pack: e.hit.pack,
                                    belongs_to: e.hit.belongs_to,
                                }}
                                components={[<b />, <b />, <b />, <b />, <b />]}
                            />
                        ) : (
                            <Trans
                                i18nKey="youSkippedHit"
                                values={{
                                    title: e.hit.title,
                                    artist: e.hit.artist,
                                    year: e.hit.year,
                                    pack: e.hit.pack,
                                }}
                                components={[<b />, <b />, <b />, <b />]}
                            />
                        ),
                } satisfies NotificationData)
            },
        )

        return () => {
            unsubscribeGuessed()
            unsubscribeHitRevealed()
            unsubscribeJoinedGame()
            unsubscribeLeftGame()
            unsubscribeNotification()
            unsubscribeSkippedHit()
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
