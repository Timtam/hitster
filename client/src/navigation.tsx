import Container from "react-bootstrap/Container"
import Nav from "react-bootstrap/Nav"
import NavDropdown from "react-bootstrap/NavDropdown"
import Navbar from "react-bootstrap/Navbar"
import { useCookies } from "react-cookie"
import { LinkContainer } from "react-router-bootstrap"
import { useNavigate } from "react-router-dom"

export default function Navigation() {
    let [cookies] = useCookies(["logged_in"])
    let navigate = useNavigate()

    return (
        <Container>
            <h2>Navigation</h2>
            <Navbar className="fixed-top" bg="light" variant="light">
                <Navbar.Collapse>
                    <Nav.Item>
                        <LinkContainer to="/">
                            <Nav.Link>Game Lobby</Nav.Link>
                        </LinkContainer>
                    </Nav.Item>
                    {cookies.logged_in !== undefined ? (
                        <NavDropdown
                            title={"Logged in as " + cookies.logged_in.username}
                        >
                            <NavDropdown.Item as="div">
                                <Nav.Link
                                    onClick={async () => {
                                        let res = await fetch(
                                            "/api/users/logout",
                                            {
                                                method: "POST",
                                                credentials: "include",
                                            },
                                        )

                                        if (res.status === 200)
                                            navigate("", { replace: true })
                                    }}
                                >
                                    Logout
                                </Nav.Link>
                            </NavDropdown.Item>
                            <NavDropdown.Item as="div">
                                <Navbar.Text>Delete account</Navbar.Text>
                            </NavDropdown.Item>
                        </NavDropdown>
                    ) : (
                        <NavDropdown title="Not logged in">
                            <NavDropdown.Item as="div">
                                <LinkContainer to="/login">
                                    <Nav.Link>Login</Nav.Link>
                                </LinkContainer>
                            </NavDropdown.Item>
                            <NavDropdown.Item as="div">
                                <LinkContainer to="/register">
                                    <Nav.Link>Register</Nav.Link>
                                </LinkContainer>
                            </NavDropdown.Item>
                        </NavDropdown>
                    )}
                </Navbar.Collapse>
            </Navbar>
        </Container>
    )
}
