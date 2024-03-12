import Container from "react-bootstrap/Container"
import Nav from "react-bootstrap/Nav"
import NavDropdown from "react-bootstrap/NavDropdown"
import Navbar from "react-bootstrap/Navbar"
import { useCookies } from "react-cookie"
import { Link } from "react-router-dom"

export default function Navigation() {
    let [cookies] = useCookies(["login"])

    return (
        <Container>
            <h2>Navigation</h2>
            <Navbar className="fixed-top" bg="light" variant="light">
                <Navbar.Collapse>
                    <Nav.Item>
                        <Nav.Link as={Link} to="/" active>
                            Game Lobby
                        </Nav.Link>
                    </Nav.Item>
                    {cookies.login !== undefined ? (
                        <Navbar.Text>Logout</Navbar.Text>
                    ) : (
                        <NavDropdown title="Not logged in">
                            <NavDropdown.Item as="div">
                                <Nav.Link as={Link} to="/login">
                                    Login
                                </Nav.Link>
                            </NavDropdown.Item>
                            <NavDropdown.Item as="div">
                                <Nav.Link as={Link} to="/register">
                                    Register
                                </Nav.Link>
                            </NavDropdown.Item>
                        </NavDropdown>
                    )}
                </Navbar.Collapse>
            </Navbar>
        </Container>
    )
}
