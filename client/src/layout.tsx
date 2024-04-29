import "bootstrap/dist/css/bootstrap.min.css"
import { useEffect, useState } from "react"
import Col from "react-bootstrap/Col"
import Container from "react-bootstrap/Container"
import Row from "react-bootstrap/Row"
import Spinner from "react-bootstrap/Spinner"
import { useCookies } from "react-cookie"
import useTimer from "react-hook-time"
import { useTranslation } from "react-i18next"
import { Outlet } from "react-router-dom"
import type { UserContext } from "./contexts"
import { User } from "./entities"
import Navigation from "./navigation"

const updateUserAuth = async () => {
    await fetch("/api/users/auth", {
        credentials: "include",
    })
}

export default function Layout() {
    let { t } = useTranslation()
    let [cookies] = useCookies(["user"])
    let [user, setUser] = useState<User | null>(null)
    let authTimer = useTimer({
        stopUpdate: true,
        onEnd: () => {
            updateUserAuth()
        },
    })

    useEffect(() => {
        if (cookies.user !== undefined) {
            try {
                let user = User.parse({
                    name: cookies.user.name,
                    id: cookies.user.id,
                    virtual: cookies.user.virtual,
                    valid_until: cookies.user.valid_until,
                })

                setUser(user)

                authTimer.setTime(user.valid_until)
                authTimer.start()
            } catch {
                setUser(null)
                updateUserAuth()
            }
        } else {
            updateUserAuth()
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
                        <Navigation user={user} />
                    </Row>
                    <Row>
                        <Col>
                            <Outlet context={{ user } satisfies UserContext} />
                        </Col>
                    </Row>
                </>
            )}
        </Container>
    )
}
