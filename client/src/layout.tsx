import EventManager from "@lomray/event-manager"
import { useLocalStorage } from "@uidotdev/usehooks"
import boolifyString from "boolify-string"
import { useEffect, useState } from "react"
import Col from "react-bootstrap/Col"
import Container from "react-bootstrap/Container"
import Row from "react-bootstrap/Row"
import Spinner from "react-bootstrap/Spinner"
import { useCookies } from "react-cookie"
import { useTranslation } from "react-i18next"
import { Outlet } from "react-router"
import usePrefersColorScheme from "use-prefers-color-scheme"
import type { Context } from "./context"
import {
    CreateGameEvent,
    ProcessHitsEvent,
    RemoveGameEvent,
    User,
} from "./entities"
import { Events, GameCreatedData, GameRemovedData } from "./events"
import ErrorModal from "./modals/error"
import WelcomeModal from "./modals/welcome"
import Navigation from "./navigation"
import NotificationPlayer from "./notification-player"
import SfxPlayer from "./sfx-player"

const updateUserAuth = async () => {
    await fetch("/api/users/auth", {
        credentials: "include",
    })
}

export default function Layout() {
    const {
        t,
        i18n: { language },
    } = useTranslation()
    const [cookies] = useCookies(["user"])
    const [user, setUser] = useState<User | null>(null)
    const [loading, setLoading] = useState(true)
    const [colorScheme] = useLocalStorage("colorScheme", "auto")
    const [welcome, setWelcome] = useLocalStorage("welcome")
    const prefersColorScheme = usePrefersColorScheme()
    const [error, setError] = useState<string | undefined>(undefined)
    const [navHeight, setNavHeight] = useState(50)

    useEffect(() => {
        let timer: ReturnType<typeof setTimeout> | null = null

        if (cookies.user !== undefined) {
            if (timer !== null) clearTimeout(timer)

            try {
                const user = User.parse({
                    name: cookies.user.name,
                    id: cookies.user.id,
                    virtual: cookies.user.virtual,
                    valid_until: cookies.user.valid_until,
                    permissions: cookies.user.permissions,
                })

                setUser(user)

                timer = setTimeout(
                    async () => {
                        setLoading(false)
                        await updateUserAuth()
                    },
                    Math.max(
                        loading ? 0 : user.valid_until.getTime() - Date.now(),
                        0,
                    ),
                )
            } catch {
                setUser(null)
                updateUserAuth()
            }
        } else {
            updateUserAuth()
        }

        return () => {
            if (timer !== null) clearTimeout(timer)
        }
    }, [cookies, loading])

    useEffect(() => {
        let eventSource: EventSource | undefined

        if (!loading) {
            eventSource = new EventSource("/api/events")

            eventSource.addEventListener("create_game", (e) => {
                EventManager.publish(Events.gameCreated, {
                    game: CreateGameEvent.parse(JSON.parse(e.data)).create_game,
                } satisfies GameCreatedData)
            })

            eventSource.addEventListener("remove_game", (e) => {
                EventManager.publish(Events.gameRemoved, {
                    game: RemoveGameEvent.parse(JSON.parse(e.data)).remove_game,
                } satisfies GameRemovedData)
            })

            eventSource.addEventListener("process_hits", (e) => {
                EventManager.publish(
                    Events.hitsProgressUpdate,
                    ProcessHitsEvent.parse(JSON.parse(e.data)).process_hits,
                )
            })
        }

        return () => {
            if (eventSource) eventSource.close()
        }
    }, [loading])

    useEffect(() => {
        document.documentElement.lang = language
        document.documentElement.dataset.bsTheme =
            colorScheme !== "auto"
                ? colorScheme
                : prefersColorScheme === "dark"
                  ? "dark"
                  : "light"
    }, [language, colorScheme, prefersColorScheme])

    useEffect(() => {
        document.body.style.paddingTop = navHeight.toString() + "px"
    }, [navHeight])

    return (
        <Container fluid className="justify-content-center">
            {user === null ? (
                <Spinner animation="border" role="status">
                    <span className="visually-hidden">{t("loading")}</span>
                </Spinner>
            ) : (
                <>
                    <Row>
                        <header>
                            <Col>
                                <NotificationPlayer user={user} />
                                <Navigation
                                    user={user}
                                    onResize={setNavHeight}
                                />
                            </Col>
                        </header>
                    </Row>
                    <Row>
                        <main>
                            <Col>
                                <SfxPlayer user={user} />
                                <WelcomeModal
                                    show={!boolifyString(welcome)}
                                    onHide={() => setWelcome("true")}
                                />
                                <ErrorModal
                                    error={error}
                                    onHide={() => setError(undefined)}
                                />
                                <Outlet
                                    context={
                                        {
                                            user,
                                            showError: (error) =>
                                                setError(error),
                                        } satisfies Context
                                    }
                                />
                            </Col>
                        </main>
                    </Row>
                </>
            )}
        </Container>
    )
}
