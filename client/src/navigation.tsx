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
        <>
            <h2 className="h4">Navigation</h2>
            <Navbar className="fixed-top" bg="light" variant="light">
                <Navbar.Collapse>
                    <Nav.Item className="me-2">
                        <LinkContainer to="/">
                            <Nav.Link>Game Lobby</Nav.Link>
                        </LinkContainer>
                    </Nav.Item>
                    {cookies.logged_in !== undefined ? (
                        <NavDropdown
                            className="me-2"
                            title={"Logged in as " + cookies.logged_in.username}
                        >
                            <NavDropdown.Item as="div" className="me-2">
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
                            <NavDropdown.Item as="div" className="me-2">
                                <Navbar.Text>Delete account</Navbar.Text>
                            </NavDropdown.Item>
                        </NavDropdown>
                    ) : (
                        <NavDropdown title="Not logged in" className="me-2">
                            <NavDropdown.Item as="div" className="me-2">
                                <LinkContainer to="/login">
                                    <Nav.Link>Login</Nav.Link>
                                </LinkContainer>
                            </NavDropdown.Item>
                            <NavDropdown.Item as="div" className="me-2">
                                <LinkContainer to="/register">
                                    <Nav.Link>Register</Nav.Link>
                                </LinkContainer>
                            </NavDropdown.Item>
                        </NavDropdown>
                    )}
                </Navbar.Collapse>
            </Navbar>
        </>
    )
}
