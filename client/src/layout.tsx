import "bootstrap/dist/css/bootstrap.min.css"
import Col from "react-bootstrap/Col"
import Container from "react-bootstrap/Container"
import Row from "react-bootstrap/Row"
import { Outlet } from "react-router-dom"
import Navigation from "./navigation"

export default function Layout() {
    return (
        <>
            <Container fluid className="justify-content-center">
                <Row>
                    <Navigation />
                </Row>
                <Row>
                    <Col>
                        <Outlet />
                    </Col>
                </Row>
            </Container>
        </>
    )
}
