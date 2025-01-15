import { useLocalStorage } from "@uidotdev/usehooks"
import boolifyString from "boolify-string"
import { useEffect, useState } from "react"
import Col from "react-bootstrap/Col"
import Container from "react-bootstrap/Container"
import Row from "react-bootstrap/Row"
import Spinner from "react-bootstrap/Spinner"
import { useCookies } from "react-cookie"
import { useTranslation } from "react-i18next"
import { Outlet } from "react-router-dom"
import usePrefersColorScheme from "use-prefers-color-scheme"
import type { Context } from "./context"
import { User } from "./entities"
import ErrorModal from "./error-modal"
import Navigation from "./navigation"
import NotificationPlayer from "./notification-player"
import { Welcome } from "./pages/welcome"
import SfxPlayer from "./sfx-player"

const updateUserAuth = async () => {
    await fetch("/api/users/auth", {
        credentials: "include",
    })
}

export default function Layout() {
    let {
        t,
        i18n: { language },
    } = useTranslation()
    let [cookies] = useCookies(["user"])
    let [user, setUser] = useState<User | null>(null)
    let [loading, setLoading] = useState(true)
    let [colorScheme] = useLocalStorage("colorScheme", "auto")
    let [welcome, setWelcome] = useLocalStorage("welcome")
    let prefersColorScheme = usePrefersColorScheme()
    let [error, setError] = useState<string | undefined>(undefined)

    useEffect(() => {
        let timer: ReturnType<typeof setTimeout> | null = null

        if (cookies.user !== undefined) {
            if (timer !== null) clearTimeout(timer)

            try {
                let user = User.parse({
                    name: cookies.user.name,
                    id: cookies.user.id,
                    virtual: cookies.user.virtual,
                    valid_until: cookies.user.valid_until,
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
    }, [cookies])

    useEffect(() => {
        document.documentElement.lang = language
        document.documentElement.dataset.bsTheme =
            colorScheme !== "auto"
                ? colorScheme
                : prefersColorScheme === "dark"
                  ? "dark"
                  : "light"
    }, [language, colorScheme, prefersColorScheme])

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
                                <Navigation user={user} />
                            </Col>
                        </header>
                    </Row>
                    <Row>
                        <main>
                            <Col>
                                <SfxPlayer user={user} />
                                <Welcome
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
