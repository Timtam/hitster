import "bootstrap/dist/css/bootstrap.min.css"
import { useEffect, useState } from "react"
import Col from "react-bootstrap/Col"
import Container from "react-bootstrap/Container"
import Row from "react-bootstrap/Row"
import { useCookies } from "react-cookie"
import { Outlet } from "react-router-dom"
import type { UserContext } from "./contexts"
import { User } from "./entities"
import Navigation from "./navigation"

export default function Layout() {
    let [cookies] = useCookies(["logged_in"])
    let [user, setUser] = useState<User | null>(null)

    useEffect(() => {
        if (cookies.logged_in !== undefined)
            setUser(
                User.parse({
                    username: cookies.logged_in.username,
                    id: cookies.logged_in.id,
                }),
            )
        else setUser(null)
    }, [cookies])

    return (
        <>
            <Container fluid className="justify-content-center">
                <Row>
                    <Navigation user={user} />
                </Row>
                <Row>
                    <Col>
                        <Outlet context={{ user } satisfies UserContext} />
                    </Col>
                </Row>
            </Container>
        </>
    )
}
