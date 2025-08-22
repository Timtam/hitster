import EventManager from "@lomray/event-manager"
import { useLocalStorage } from "@uidotdev/usehooks"
import { useEffect, useRef, useState } from "react"
import Container from "react-bootstrap/Container"
import Nav from "react-bootstrap/Nav"
import NavDropdown from "react-bootstrap/NavDropdown"
import Navbar from "react-bootstrap/Navbar"
import Overlay from "react-bootstrap/Overlay"
import Popover from "react-bootstrap/Popover"
import { Trans, useTranslation } from "react-i18next"
import {
    BsDatabaseCheck,
    BsDatabaseExclamation,
    BsMenuButtonFill,
} from "react-icons/bs"
import { NavLink, useNavigate } from "react-router"
import { User } from "./entities"
import { Events, HitsProgressUpdateData } from "./events"
import SettingsModal from "./modals/settings"
import ShortcutsModal from "./modals/shortcuts"

export default function Navigation({
    user,
    onResize,
}: {
    user: User | null
    onResize: (size: number) => void
}) {
    const navRef = useRef<any | null>(null)
    const navigate = useNavigate()
    const { t } = useTranslation()
    const [showSettings, setShowSettings] = useState(false)
    const [showShortcuts, setShowShortcuts] = useState(false)
    const [showHitsStatus, setShowHitsStatus] = useState(false)
    const [hitsStatus, setHitsStatus] = useState<HitsProgressUpdateData>({
        available: 0,
        downloading: 0,
        processing: 0,
    })
    const [, setWelcome] = useLocalStorage("welcome")
    const [popoverTarget, setPopoverTarget] = useState<any>(null)
    const popoverRef = useRef(null)

    const handleResize = () => {
        onResize(navRef.current!.offsetHeight)
    }

    useEffect(() => {
        window.addEventListener("resize", handleResize)
        const unsubscribeHitsStatus = EventManager.subscribe(
            Events.hitsProgressUpdate,
            (e: HitsProgressUpdateData) => {
                setHitsStatus(e)
            },
        )
        return () => {
            window.removeEventListener("resize", handleResize)
            unsubscribeHitsStatus()
        }
    }, [])

    return (
        <>
            <Navbar
                collapseOnSelect
                aria-label={t("navigation")}
                fixed="top"
                variant="light"
                style={{ margin: "0px", padding: "0px" }}
                ref={navRef}
                bg="primary"
                data-bs-theme="dark"
            >
                <Container fluid>
                    <Navbar.Toggle />
                    <Navbar.Collapse>
                        <Nav
                            activeKey={location.pathname}
                            navbarScroll
                            style={{ maxHeight: "100px" }}
                        >
                            <NavDropdown
                                className="me-2"
                                title={
                                    <BsMenuButtonFill
                                        size="2em"
                                        title={t("mainMenu")}
                                    />
                                }
                            >
                                <Nav.Item>
                                    <Nav.Link as={NavLink} eventKey="/" to="/">
                                        {t("gameLobby")}
                                    </Nav.Link>
                                </Nav.Item>
                                <Nav.Item>
                                    <Nav.Link
                                        as={NavLink}
                                        eventKey="/hits"
                                        to="/hits"
                                    >
                                        {t("browseHits")}
                                    </Nav.Link>
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
                        <Nav ref={popoverRef}>
                            <Nav.Item>
                                <Nav.Link
                                    aria-expanded={showHitsStatus}
                                    onClick={(e) => {
                                        setShowHitsStatus(!showHitsStatus)
                                        setPopoverTarget(e.target as any)
                                    }}
                                >
                                    {hitsStatus.downloading > 0 ||
                                    hitsStatus.processing > 0 ? (
                                        <BsDatabaseExclamation
                                            title={t(
                                                "hitStatusProcessingTitle",
                                            )}
                                            size="2em"
                                        />
                                    ) : (
                                        <BsDatabaseCheck
                                            title={t("hitStatusFinishedTitle")}
                                            size="2em"
                                        />
                                    )}
                                </Nav.Link>
                            </Nav.Item>
                            <Overlay
                                show={showHitsStatus}
                                target={popoverTarget}
                                placement="bottom"
                                container={popoverRef}
                                containerPadding={20}
                                rootClose
                                onHide={() => setShowHitsStatus(false)}
                            >
                                <Popover id="hits-status-popover">
                                    <Popover.Header as="h3">
                                        {hitsStatus.downloading > 0 ||
                                        hitsStatus.processing > 0
                                            ? t("hitStatusProcessingTitle")
                                            : t("hitStatusFinishedTitle")}
                                    </Popover.Header>
                                    <Popover.Body>
                                        {hitsStatus.downloading > 0 ||
                                        hitsStatus.processing > 0
                                            ? t("hitStatusProcessingMessage", {
                                                  downloading:
                                                      hitsStatus.downloading.toString() +
                                                      " " +
                                                      t("downloading", {
                                                          count: hitsStatus.downloading,
                                                          hits: t("hit", {
                                                              count: hitsStatus.downloading,
                                                          }),
                                                      }),
                                                  processing:
                                                      hitsStatus.processing.toString() +
                                                      " " +
                                                      t("processing", {
                                                          count: hitsStatus.processing,
                                                          hits: t("hit", {
                                                              count: hitsStatus.processing,
                                                          }),
                                                      }),
                                                  downloaded:
                                                      hitsStatus.available.toString() +
                                                      " " +
                                                      t("downloaded", {
                                                          count: hitsStatus.available,
                                                          hits: t("hit", {
                                                              count: hitsStatus.available,
                                                          }),
                                                      }),
                                              })
                                            : t("hitStatusFinishedMessage", {
                                                  count: hitsStatus.available,
                                                  hits:
                                                      hitsStatus.available.toString() +
                                                      " " +
                                                      t("hit", {
                                                          count: hitsStatus.available,
                                                      }),
                                              })}
                                    </Popover.Body>
                                </Popover>
                            </Overlay>
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
                                        <Nav.Link as={NavLink} to="/login">
                                            {t("login")}
                                        </Nav.Link>
                                    </NavDropdown.Item>
                                    <NavDropdown.Item as="div" className="me-2">
                                        <Nav.Link as={NavLink} to="/register">
                                            {t("register")}
                                        </Nav.Link>
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
