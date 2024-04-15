import { useMemo } from "react"
import Button from "react-bootstrap/Button"
import Table from "react-bootstrap/Table"
import { Helmet } from "react-helmet-async"
import { useTranslation } from "react-i18next"
import { Link, useLoaderData, useNavigate } from "react-router-dom"
import { useUser } from "../contexts"
import { Game, GameState } from "../entities"
import { useRevalidateOnInterval } from "../hooks"
import GameService from "../services/games.service"

export async function loader(): Promise<Game[]> {
    let gs = new GameService()
    return await gs.getAll()
}

export function Lobby() {
    let gameService = useMemo(() => new GameService(), [])
    let { user } = useUser()
    let games = useLoaderData() as Game[]
    let navigate = useNavigate()
    let { t } = useTranslation()

    useRevalidateOnInterval({ enabled: true, interval: 5000 })

    const createGame = async () => {
        let game = await gameService.create()
        navigate("/game/" + game.id)
    }

    return (
        <>
            <Helmet>
                <title>{t("gameLobby")} - Hitster</title>
            </Helmet>
            <Button disabled={user === null} onClick={createGame}>
                {user === null
                    ? t("createNewGameNotLoggedIn")
                    : t("createNewGame")}
            </Button>
            <Table responsive>
                <thead>
                    <tr>
                        <th>{t("gameId")}</th>
                        <th>{t("player", { count: 2 })}</th>
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
