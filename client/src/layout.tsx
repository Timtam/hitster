import EventManager from "@lomray/event-manager"
import { useLocalStorage } from "@uidotdev/usehooks"
import boolifyString from "boolify-string"
import { useEffect, useRef, useState } from "react"
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
    CreateHitIssueEvent,
    DeleteHitIssueEvent,
    ProcessHitsEvent,
    RemoveGameEvent,
    User,
} from "./entities"
import {
    Events,
    GameCreatedData,
    GameRemovedData,
    IssueCreatedData,
    IssueDeletedData,
} from "./events"
import ErrorModal from "./modals/error"
import WelcomeModal from "./modals/welcome"
import Navigation from "./navigation"
import NotificationPlayer from "./notification-player"
import SfxPlayer from "./sfx-player"
import { refreshUserAuth } from "./user-auth"

function hasSamePermissions(a: User["permissions"], b: User["permissions"]) {
    const keys = Object.keys(a) as Array<keyof User["permissions"]>
    return keys.every((key) => a[key] === b[key])
}

function hasSameUserIdentity(a: User, b: User) {
    return (
        a.id === b.id &&
        a.name === b.name &&
        a.virtual === b.virtual &&
        hasSamePermissions(a.permissions, b.permissions)
    )
}

export default function Layout() {
    const {
        t,
        i18n: { language },
    } = useTranslation()
    const [cookies] = useCookies(["user"])
    const [user, setUser] = useState<User | null>(null)
    const [userValidUntil, setUserValidUntil] = useState<number | null>(null)
    const [colorScheme] = useLocalStorage("colorScheme", "auto")
    const [welcome, setWelcome] = useLocalStorage("welcome")
    const prefersColorScheme = usePrefersColorScheme()
    const [error, setError] = useState<string | undefined>(undefined)
    const [navHeight, setNavHeight] = useState(50)
    const hasValidatedStartupAuth = useRef(false)

    useEffect(() => {
        if (hasValidatedStartupAuth.current || cookies.user === undefined)
            return

        hasValidatedStartupAuth.current = true
        void refreshUserAuth()
    }, [cookies.user])

    useEffect(() => {
        if (cookies.user !== undefined) {
            try {
                const nextUser = User.parse({
                    name: cookies.user.name,
                    id: cookies.user.id,
                    virtual: cookies.user.virtual,
                    valid_until: cookies.user.valid_until,
                    permissions: cookies.user.permissions,
                })

                setUserValidUntil(nextUser.valid_until.getTime())
                setUser((current) =>
                    current !== null && hasSameUserIdentity(current, nextUser)
                        ? current
                        : nextUser,
                )
            } catch {
                setUser(null)
                setUserValidUntil(null)
                void refreshUserAuth()
            }
        } else {
            setUser(null)
            setUserValidUntil(null)
            void refreshUserAuth()
        }
    }, [cookies.user])

    useEffect(() => {
        if (userValidUntil === null) return

        const timer = setTimeout(
            () => {
                void refreshUserAuth()
            },
            Math.max(userValidUntil - Date.now(), 0),
        )

        return () => {
            clearTimeout(timer)
        }
    }, [userValidUntil])

    useEffect(() => {
        let eventSource: EventSource | undefined

        if (user !== null) {
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

            eventSource.addEventListener("create_hit_issue", (e) => {
                EventManager.publish(Events.issueCreated, {
                    issue: CreateHitIssueEvent.parse(JSON.parse(e.data))
                        .create_hit_issue,
                } satisfies IssueCreatedData)
            })

            eventSource.addEventListener("delete_hit_issue", (e) => {
                const data = DeleteHitIssueEvent.parse(
                    JSON.parse(e.data),
                ).delete_hit_issue
                EventManager.publish(Events.issueDeleted, {
                    hitId: data.hit_id,
                    issueId: data.issue_id,
                } satisfies IssueDeletedData)
            })
        }

        return () => {
            if (eventSource) eventSource.close()
        }
    }, [user])

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
