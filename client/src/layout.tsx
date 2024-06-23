import { useLocalStorage } from "@uidotdev/usehooks"
import boolifyString from "boolify-string"
import "bootstrap/dist/css/bootstrap.min.css"
import { useEffect, useState } from "react"
import Col from "react-bootstrap/Col"
import Container from "react-bootstrap/Container"
import Row from "react-bootstrap/Row"
import Spinner from "react-bootstrap/Spinner"
import { useCookies } from "react-cookie"
import { Helmet } from "react-helmet-async"
import { Trans, useTranslation } from "react-i18next"
import { Outlet } from "react-router-dom"
import type { Context } from "./context"
import { User } from "./entities"
import Navigation from "./navigation"
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
    let [welcome, setWelcome] = useLocalStorage("welcome")

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

    return (
        <>
            <Helmet>
                <html lang={language} />
            </Helmet>
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
                                    <Outlet
                                        context={
                                            {
                                                user,
                                            } satisfies Context
                                        }
                                    />
                                </Col>
                            </main>
                        </Row>
                        <Row>
                            <footer>
                                <Col>
                                    <p>
                                        &copy;{" "}
                                        {new Date().getFullYear() !== 2024
                                            ? `2024 - ${new Date().getFullYear()}`
                                            : "2024"}{" "}
                                        Toni Barth &amp; Friends.{" "}
                                        {t("allRightsReserved")}.
                                    </p>
                                    <p>
                                        <Trans
                                            i18nKey="sourceCodeAvailableAt"
                                            components={[
                                                <a
                                                    href="https://github.com/Timtam/hitster"
                                                    target="_blank"
                                                />,
                                            ]}
                                        />
                                    </p>
                                    <p>
                                        {t("version", {
                                            clientVersion: "__CLIENT_VERSION__",
                                            serverVersion: "__SERVER_VERSION__",
                                        })}
                                    </p>
                                    <p>
                                        <Trans
                                            i18nKey="ownedBy"
                                            components={[
                                                <a
                                                    href="https://hitstergame.com/"
                                                    target="_blank"
                                                />,
                                            ]}
                                        />
                                    </p>
                                    <p>
                                        <Trans
                                            i18nKey="issue"
                                            components={[
                                                <a
                                                    href="https://github.com/Timtam/hitster/issues/new"
                                                    target="_blank"
                                                />,
                                            ]}
                                        />
                                    </p>
                                </Col>
                            </footer>
                        </Row>
                    </>
                )}
            </Container>
        </>
    )
}
