import { useLocalStorage } from "@uidotdev/usehooks"
import { useState } from "react"
import Container from "react-bootstrap/Container"
import Nav from "react-bootstrap/Nav"
import NavDropdown from "react-bootstrap/NavDropdown"
import Navbar from "react-bootstrap/Navbar"
import { Trans, useTranslation } from "react-i18next"
import { LinkContainer } from "react-router-bootstrap"
import { useNavigate } from "react-router-dom"
import { User } from "./entities"
import SettingsModal from "./modals/settings"
import ShortcutsModal from "./modals/shortcuts"

export default function Navigation({ user }: { user: User | null }) {
    const navigate = useNavigate()
    const { t } = useTranslation()
    const [showSettings, setShowSettings] = useState(false)
    const [showShortcuts, setShowShortcuts] = useState(false)
    const [, setWelcome] = useLocalStorage("welcome")

    return (
        <>
            <Navbar
                aria-label={t("navigation")}
                fixed="top"
                variant="light"
                style={{ margin: "0px", padding: "0px" }}
            >
                <Container fluid>
                    <Navbar.Toggle />
                    <Navbar.Collapse>
                        <Nav
                            activeKey={location.pathname}
                            navbarScroll
                            style={{ maxHeight: "100px" }}
                        >
                            <NavDropdown className="me-2" title={t("mainMenu")}>
                                <Nav.Item
                                    aria-current={location.pathname === "/"}
                                >
                                    <LinkContainer to="/">
                                        <Nav.Link>{t("gameLobby")}</Nav.Link>
                                    </LinkContainer>
                                </Nav.Item>
                                <Nav.Item>
                                    <Nav.Link
                                        aria-expanded="false"
                                        onClick={() => setWelcome("false")}
                                    >
                                        {t("welcome")}
                                    </Nav.Link>
                                </Nav.Item>
                                <Nav.Item>
                                    <Nav.Link
                                        aria-expanded="false"
                                        onClick={() => setShowShortcuts(true)}
                                    >
                                        {t("keyboardShortcut", { count: 2 })}
                                    </Nav.Link>
                                </Nav.Item>
                                <Nav.Item>
                                    <Nav.Link
                                        aria-expanded="false"
                                        onClick={() => setShowSettings(true)}
                                    >
                                        {t("settings")}
                                    </Nav.Link>
                                </Nav.Item>
                                <NavDropdown.Divider />
                                <p className="mb-3 px-2 fs-6 text-nowrap">
                                    &copy;{" "}
                                    {new Date().getFullYear() !== 2024
                                        ? `2024 - ${new Date().getFullYear()}`
                                        : "2024"}{" "}
                                    Toni Barth &amp; Friends.{" "}
                                    {t("allRightsReserved")}.
                                </p>
                                <p className="mb-3 px-2 fs-6 text-nowrap">
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
                                <p className="mb-6 px-2 fs-6 text-nowrap">
                                    {t("version", {
                                        version: "__VERSION__",
                                    })}{" "}
                                    (
                                    <a href="https://github.com/Timtam/hitster/blob/__BRANCH__/CHANGELOG.md">
                                        {t("changelog")}
                                    </a>
                                    )
                                </p>
                                <p className="mb-3 px-2 fs-6 text-nowrap">
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
                                <p className="mb-3 px-2 fs-6 text-nowrap">
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
                            </NavDropdown>
                        </Nav>
                        <Nav className="me-auto"></Nav>
                        <Nav>
                            {user?.virtual === false ? (
                                <NavDropdown
                                    title={t("loggedInAs", {
                                        username: user?.name,
                                    })}
                                >
                                    <NavDropdown.Item as="div" className="me-2">
                                        <Nav.Link
                                            onClick={async () => {
                                                const res = await fetch(
                                                    "/api/users/logout",
                                                    {
                                                        method: "POST",
                                                        credentials: "include",
                                                    },
                                                )

                                                if (res.status === 200)
                                                    navigate("", {
                                                        replace: true,
                                                    })
                                            }}
                                        >
                                            {t("logout")}
                                        </Nav.Link>
                                    </NavDropdown.Item>
                                    <NavDropdown.Item as="div" className="me-2">
                                        <Navbar.Text>
                                            {t("deleteAccount")}
                                        </Navbar.Text>
                                    </NavDropdown.Item>
                                </NavDropdown>
                            ) : (
                                <NavDropdown
                                    title={t("knownAs", {
                                        username: user?.name,
                                    })}
                                    className="me-auto"
                                >
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
                        </Nav>{" "}
                    </Navbar.Collapse>
                </Container>
            </Navbar>
            <SettingsModal
                show={showSettings}
                onHide={() => setShowSettings(false)}
            />
            <ShortcutsModal
                show={showShortcuts}
                onHide={() => setShowShortcuts(false)}
            />
        </>
    )
}
