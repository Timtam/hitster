import "bootstrap/dist/css/bootstrap.min.css"
import { useEffect, useState } from "react"
import Col from "react-bootstrap/Col"
import Container from "react-bootstrap/Container"
import Row from "react-bootstrap/Row"
import Spinner from "react-bootstrap/Spinner"
import { useCookies } from "react-cookie"
import { useTranslation } from "react-i18next"
import { Outlet } from "react-router-dom"
import type { Context } from "./context"
import { User } from "./entities"
import Navigation from "./navigation"
import SfxPlayer from "./sfx-player"

const updateUserAuth = async () => {
    await fetch("/api/users/auth", {
        credentials: "include",
    })
}

export default function Layout() {
    let { t } = useTranslation()
    let [cookies] = useCookies(["user"])
    let [user, setUser] = useState<User | null>(null)
    let [loading, setLoading] = useState(true)

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
        <Container fluid className="justify-content-center">
            {user === null ? (
                <Spinner animation="border" role="status">
                    <span className="visually-hidden">{t("loading")}</span>
                </Spinner>
            ) : (
                <>
                    <Row>
                        <Col>
                            <Navigation user={user} />
                        </Col>
                    </Row>
                    <Row>
                        <Col>
                            <SfxPlayer user={user} />
                            <Outlet
                                context={
                                    {
                                        user,
                                    } satisfies Context
                                }
                            />
                        </Col>
                    </Row>
                </>
            )}
        </Container>
    )
}
