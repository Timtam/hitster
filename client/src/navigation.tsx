import Nav from "react-bootstrap/Nav"
import NavDropdown from "react-bootstrap/NavDropdown"
import Navbar from "react-bootstrap/Navbar"
import { useCookies } from "react-cookie"
import { useTranslation } from "react-i18next"
import { LinkContainer } from "react-router-bootstrap"
import { useNavigate } from "react-router-dom"

export default function Navigation() {
    let [cookies] = useCookies(["logged_in"])
    let navigate = useNavigate()
    const {
        t,
        i18n: { changeLanguage, language, services },
    } = useTranslation()

    return (
        <>
            <h2 className="h4">{t("navigationHeading")}</h2>
            <Navbar className="fixed-top" bg="light" variant="light">
                <Navbar.Collapse>
                    <Nav.Item className="me-2">
                        <LinkContainer to="/">
                            <Nav.Link>{t("gameLobby")}</Nav.Link>
                        </LinkContainer>
                    </Nav.Item>
                    {cookies.logged_in !== undefined ? (
                        <NavDropdown
                            className="me-2"
                            title={t("loggedInAs", {
                                username: cookies.logged_in.username,
                            })}
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
                                    {t("logout")}
                                </Nav.Link>
                            </NavDropdown.Item>
                            <NavDropdown.Item as="div" className="me-2">
                                <Navbar.Text>{t("deleteAccount")}</Navbar.Text>
                            </NavDropdown.Item>
                        </NavDropdown>
                    ) : (
                        <NavDropdown title={t("notLoggedIn")} className="me-2">
                            <NavDropdown.Item as="div" className="me-2">
                                <LinkContainer to="/login">
                                    <Nav.Link>{t("login")}</Nav.Link>
                                </LinkContainer>
                            </NavDropdown.Item>
                            <NavDropdown.Item as="div" className="me-2">
                                <LinkContainer to="/register">
                                    <Nav.Link>{t("register")}</Nav.Link>
                                </LinkContainer>
                            </NavDropdown.Item>
                        </NavDropdown>
                    )}
                    <NavDropdown
                        className="me-2"
                        title={t("language", {
                            language: new Intl.DisplayNames(language, {
                                type: "language",
                            }).of(language),
                        })}
                    >
                        {Object.keys(services.resourceStore.data).map(
                            (langcode) => {
                                let nameGenerator = new Intl.DisplayNames(
                                    langcode,
                                    { type: "language" },
                                )
                                let displayName = nameGenerator.of(langcode)
                                return (
                                    <NavDropdown.Item as="div" className="me-2">
                                        <Nav.Link
                                            onClick={() => {
                                                changeLanguage(langcode)
                                            }}
                                        >
                                            {displayName}
                                        </Nav.Link>
                                    </NavDropdown.Item>
                                )
                            },
                        )}
                    </NavDropdown>
                </Navbar.Collapse>
            </Navbar>
        </>
    )
}
