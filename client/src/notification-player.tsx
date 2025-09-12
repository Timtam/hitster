import EventManager from "@lomray/event-manager"
import { ReactNode, useCallback, useEffect, useRef, useState } from "react"
import { flushSync } from "react-dom"
import { createRoot } from "react-dom/client"
import { Trans, useTranslation } from "react-i18next"
import { ToastContainer, toast } from "react-toastify"
import { User } from "./entities"
import {
    ClaimedHitData,
    Events,
    GuessedData,
    HitRevealedData,
    JoinedGameData,
    LeftGameData,
    NotificationData,
    SkippedHitData,
    TokenReceivedData,
} from "./events"

interface SpeechEvent {
    text: string
}

const TIMER_DURATION: number = 150

export default function NotificationPlayer({ user }: { user: User | null }) {
    const { t } = useTranslation()
    const [politeness, setPoliteness] = useState<"polite" | "assertive">(
        "polite",
    )
    const output = useRef<HTMLParagraphElement | null>(null)
    const events = useRef<SpeechEvent[]>([])
    const timer = useRef<ReturnType<typeof setTimeout> | null>(null)

    const nodeToString = (node: ReactNode) => {
        const div = document.createElement("div")
        const root = createRoot(div)
        flushSync(() => root.render(node))
        const text = div.innerText
        root.unmount()
        return text
    }

    const handleSpeechEvent = useCallback(() => {
        if (events.current.length === 0) {
            if (output.current) output.current.innerHTML = ""
            timer.current = null
            return
        }

        if (output.current) output.current.innerHTML = events.current[0].text
        events.current.shift()
        timer.current = setTimeout(handleSpeechEvent, TIMER_DURATION)
    }, [])

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
                if (e.toast !== false)
                    toast(
                        <div className="p-3" aria-hidden={true}>
                            {e.text}
                        </div>,
                    )
                if (timer.current === null) {
                    handleSpeechEvent()
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

        const unsubscribeTokenReceived = EventManager.subscribe(
            Events.tokenReceived,
            (e: TokenReceivedData) => {
                EventManager.publish(Events.notification, {
                    toast: false,
                    text:
                        e.player.id !== user?.id
                            ? t("otherReceivedToken", { player: e.player.name })
                            : t("youReceivedToken"),
                } satisfies NotificationData)
            },
        )

        const unsubscribeHitRevealed = EventManager.subscribe(
            Events.hitRevealed,
            (e: HitRevealedData) => {
                EventManager.publish(Events.notification, {
                    toast: false,
                    interruptTts: true,
                    text:
                        e.hit.belongs_to !== "" ? (
                            <Trans
                                i18nKey="hitRevealedBelonging"
                                values={{
                                    title: e.hit.title,
                                    artist: e.hit.artist,
                                    year: e.hit.year,
                                    belongs_to: e.hit.belongs_to,
                                    player: e.player?.name ?? t("noone"),
                                }}
                                components={[
                                    <b />,
                                    <a
                                        href={`/hits/${e.hit.id}`}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                    >
                                        <b />
                                    </a>,
                                    <b />,
                                    <b />,
                                    <b />,
                                ]}
                                shouldUnescape={true}
                                tOptions={{
                                    interpolation: { escapeValue: true },
                                }}
                            />
                        ) : (
                            <Trans
                                i18nKey="hitRevealed"
                                values={{
                                    title: e.hit.title,
                                    artist: e.hit.artist,
                                    year: e.hit.year,
                                    player: e.player?.name ?? t("noone"),
                                }}
                                components={[
                                    <b />,
                                    <a
                                        href={`/hits/${e.hit.id}`}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                    >
                                        <b />
                                    </a>,
                                    <b />,
                                    <b />,
                                ]}
                                shouldUnescape={true}
                                tOptions={{
                                    interpolation: { escapeValue: true },
                                }}
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
                        toast: false,
                        text: t("guessNothing", { player: e.player.name }),
                    } satisfies NotificationData)
                else
                    EventManager.publish(Events.notification, {
                        toast: false,
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
                                        belongs_to: e.hit.belongs_to,
                                        player: e.player?.name ?? t("noone"),
                                    }}
                                    components={[
                                        <b />,
                                        <a
                                            href={`/hits/${e.hit.id}`}
                                            target="_blank"
                                            rel="noopener noreferrer"
                                        >
                                            <b />
                                        </a>,
                                        <b />,
                                        <b />,
                                        <b />,
                                    ]}
                                    shouldUnescape={true}
                                    tOptions={{
                                        interpolation: { escapeValue: true },
                                    }}
                                />
                            ) : (
                                <Trans
                                    i18nKey="otherSkippedHit"
                                    values={{
                                        title: e.hit.title,
                                        artist: e.hit.artist,
                                        year: e.hit.year,
                                        player: e.player?.name ?? t("noone"),
                                    }}
                                    components={[
                                        <b />,
                                        <a
                                            href={`/hits/${e.hit.id}`}
                                            target="_blank"
                                            rel="noopener noreferrer"
                                        >
                                            <b />
                                        </a>,
                                        <b />,
                                        <b />,
                                    ]}
                                    shouldUnescape={true}
                                    tOptions={{
                                        interpolation: { escapeValue: true },
                                    }}
                                />
                            )
                        ) : e.hit.belongs_to !== "" ? (
                            <Trans
                                i18nKey="youSkippedHitBelonging"
                                values={{
                                    title: e.hit.title,
                                    artist: e.hit.artist,
                                    year: e.hit.year,
                                    belongs_to: e.hit.belongs_to,
                                }}
                                components={[
                                    <a
                                        href={`/hits/${e.hit.id}`}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                    >
                                        <b />
                                    </a>,
                                    <b />,
                                    <b />,
                                    <b />,
                                ]}
                                shouldUnescape={true}
                                tOptions={{
                                    interpolation: { escapeValue: true },
                                }}
                            />
                        ) : (
                            <Trans
                                i18nKey="youSkippedHit"
                                values={{
                                    title: e.hit.title,
                                    artist: e.hit.artist,
                                    year: e.hit.year,
                                }}
                                components={[
                                    <a
                                        href={`/hits/${e.hit.id}`}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                    >
                                        <b />
                                    </a>,
                                    <b />,
                                    <b />,
                                ]}
                                shouldUnescape={true}
                                tOptions={{
                                    interpolation: { escapeValue: true },
                                }}
                            />
                        ),
                } satisfies NotificationData)
            },
        )

        const unsubscribeClaimedHit = EventManager.subscribe(
            Events.claimedHit,
            (e: ClaimedHitData) => {
                EventManager.publish(Events.notification, {
                    text:
                        e.player.id !== user?.id ? (
                            e.hit.belongs_to !== "" ? (
                                <Trans
                                    i18nKey="otherClaimedHitBelonging"
                                    values={{
                                        title: e.hit.title,
                                        artist: e.hit.artist,
                                        year: e.hit.year,
                                        belongs_to: e.hit.belongs_to,
                                        player: e.player?.name ?? t("noone"),
                                    }}
                                    components={[
                                        <b />,
                                        <a
                                            href={`/hits/${e.hit.id}`}
                                            target="_blank"
                                            rel="noopener noreferrer"
                                        >
                                            <b />
                                        </a>,
                                        <b />,
                                        <b />,
                                        <b />,
                                    ]}
                                    shouldUnescape={true}
                                    tOptions={{
                                        interpolation: { escapeValue: true },
                                    }}
                                />
                            ) : (
                                <Trans
                                    i18nKey="otherClaimedHit"
                                    values={{
                                        title: e.hit.title,
                                        artist: e.hit.artist,
                                        year: e.hit.year,
                                        player: e.player?.name ?? t("noone"),
                                    }}
                                    components={[
                                        <b />,
                                        <a
                                            href={`/hits/${e.hit.id}`}
                                            target="_blank"
                                            rel="noopener noreferrer"
                                        >
                                            <b />
                                        </a>,
                                        <b />,
                                        <b />,
                                    ]}
                                    shouldUnescape={true}
                                    tOptions={{
                                        interpolation: { escapeValue: true },
                                    }}
                                />
                            )
                        ) : e.hit.belongs_to !== "" ? (
                            <Trans
                                i18nKey="youClaimedHitBelonging"
                                values={{
                                    title: e.hit.title,
                                    artist: e.hit.artist,
                                    year: e.hit.year,
                                    belongs_to: e.hit.belongs_to,
                                }}
                                components={[
                                    <a
                                        href={`/hits/${e.hit.id}`}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                    >
                                        <b />
                                    </a>,
                                    <b />,
                                    <b />,
                                    <b />,
                                ]}
                                shouldUnescape={true}
                                tOptions={{
                                    interpolation: { escapeValue: true },
                                }}
                            />
                        ) : (
                            <Trans
                                i18nKey="youClaimedHit"
                                values={{
                                    title: e.hit.title,
                                    artist: e.hit.artist,
                                    year: e.hit.year,
                                }}
                                components={[
                                    <a
                                        href={`/hits/${e.hit.id}`}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                    >
                                        <b />
                                    </a>,
                                    <b />,
                                    <b />,
                                ]}
                                shouldUnescape={true}
                                tOptions={{
                                    interpolation: { escapeValue: true },
                                }}
                            />
                        ),
                } satisfies NotificationData)
            },
        )

        return () => {
            unsubscribeClaimedHit()
            unsubscribeGuessed()
            unsubscribeHitRevealed()
            unsubscribeJoinedGame()
            unsubscribeLeftGame()
            unsubscribeNotification()
            unsubscribeSkippedHit()
            unsubscribeTokenReceived()
        }
    }, [handleSpeechEvent, t, user])

    return (
        <>
            <div aria-hidden={true}>
                <ToastContainer
                    position="top-right"
                    autoClose={false}
                    hideProgressBar={true}
                    newestOnTop={true}
                    closeOnClick={false}
                    pauseOnFocusLoss={false}
                    role=""
                    closeButton
                />
            </div>
            <p
                aria-live={politeness}
                aria-atomic={true}
                ref={output}
                className="visually-hidden"
            />
        </>
    )
}
