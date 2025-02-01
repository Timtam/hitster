import EventManager from "@lomray/event-manager"
import {
    bindKeyCombo,
    BrowserKeyComboEvent,
    unbindKeyCombo,
} from "@rwh/keystrokes"
import { detect } from "detect-browser"
import { useCallback, useEffect, useMemo } from "react"
import Dropdown from "react-bootstrap/Dropdown"
import Table from "react-bootstrap/Table"
import { Helmet } from "react-helmet-async"
import { useTranslation } from "react-i18next"
import { Link, useLoaderData, useNavigate } from "react-router-dom"
import { useContext } from "../context"
import { Game, GameMode, GameState } from "../entities"
import { Events, JoinedGameData } from "../events"
import { useModalShown, useRevalidateOnInterval } from "../hooks"
import GameService from "../services/games.service"

export default function Lobby() {
    const gameService = useMemo(() => new GameService(), [])
    const { user } = useContext()
    const games = useLoaderData() as Game[]
    const navigate = useNavigate()
    const { t } = useTranslation()
    const modalShown = useModalShown()

    useRevalidateOnInterval({ enabled: true, interval: 5000 })

    const createGame = useCallback(
        async (mode: GameMode) => {
            const game = await gameService.create(mode)
            EventManager.publish(Events.joinedGame, {
                player: null,
            } satisfies JoinedGameData)
            navigate("/game/" + game.id)
        },
        [gameService, navigate],
    )

    useEffect(() => {
        const handleNewPublicGame = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()
                createGame(GameMode.Public)
            },
        }

        const handleNewPrivateGame = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()
                createGame(GameMode.Private)
            },
        }

        const handleNewLocalGame = {
            onPressed: (e: BrowserKeyComboEvent) => {
                e.finalKeyEvent.preventDefault()
                createGame(GameMode.Local)
            },
        }

        if (!modalShown) {
            bindKeyCombo("alt + shift + u", handleNewPublicGame)
            bindKeyCombo("alt + shift + r", handleNewPrivateGame)
            bindKeyCombo("alt + shift + l", handleNewLocalGame)
        }

        return () => {
            unbindKeyCombo("alt + shift + u", handleNewPublicGame)
            unbindKeyCombo("alt + shift + r", handleNewPrivateGame)
            unbindKeyCombo("alt + shift + l", handleNewLocalGame)
        }
    }, [createGame, modalShown])

    return (
        <>
            <Helmet>
                <title>{t("gameLobby")} - Hitster</title>
            </Helmet>
            <h2>{t("gameLobby")}</h2>
            <Dropdown>
                <Dropdown.Toggle variant="success" disabled={user === null}>
                    {t("createNewGame")}
                </Dropdown.Toggle>
                <Dropdown.Menu>
                    <Dropdown.Item
                        onClick={() => createGame(GameMode.Public)}
                        aria-keyshortcuts={t("publicGameShortcut")}
                        aria-label={
                            detect()?.name === "firefox"
                                ? `${t("publicGameShortcut")} ${t("publicGame")}`
                                : ""
                        }
                    >
                        {t("publicGame")}
                    </Dropdown.Item>
                    <Dropdown.Item
                        onClick={() => createGame(GameMode.Private)}
                        aria-keyshortcuts={t("privateGameShortcut")}
                        aria-label={
                            detect()?.name === "firefox"
                                ? `${t("privateGameShortcut")} ${t("privateGame")}`
                                : ""
                        }
                    >
                        {t("privateGame")}
                    </Dropdown.Item>
                    <Dropdown.Item
                        onClick={() => createGame(GameMode.Local)}
                        aria-keyshortcuts={t("localGameShortcut")}
                        aria-label={
                            detect()?.name === "firefox"
                                ? `${t("localGameShortcut")} ${t("localGame")}`
                                : ""
                        }
                    >
                        {t("localGame")}
                    </Dropdown.Item>
                </Dropdown.Menu>
            </Dropdown>
            <Table responsive>
                <thead>
                    <tr>
                        <th>{t("gameId")}</th>
                        <th>{t("player", { count: 2 })}</th>
                        <th>{t("mode")}</th>
                        <th>{t("state")}</th>
                    </tr>
                </thead>
                <tbody>
                    {games.map((game) => {
                        return (
                            <tr>
                                <td>
                                    <Link to={"/game/" + game.id}>
                                        {game.id}
                                    </Link>
                                </td>
                                <td>{game.players.length}</td>
                                <td>{t(game.mode)}</td>
                                <td>
                                    {game.state === GameState.Open
                                        ? t("open")
                                        : t("running")}
                                </td>
                            </tr>
                        )
                    })}
                </tbody>
            </Table>
        </>
    )
}
