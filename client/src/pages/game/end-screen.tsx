import EventManager from "@lomray/event-manager"
import { useEffect } from "react"
import Modal from "react-bootstrap/Modal"
import Table from "react-bootstrap/Table"
import { useTranslation } from "react-i18next"
import { useImmer } from "use-immer"
import { useContext } from "../../context"
import type { Game, Player } from "../../entities"
import { Events, Sfx } from "../../events"
import { isSlotCorrect } from "./utils"

export default ({
    game,
    show,
    onHide,
}: {
    game: Game
    show: boolean
    onHide: () => void
}) => {
    let { user } = useContext()
    let { t } = useTranslation()
    let [winner, setWinner] = useImmer<Player | undefined>(undefined)

    useEffect(() => {
        setWinner(
            game.players.find(
                (p) =>
                    p.hits.length === game.goal - 1 &&
                    isSlotCorrect(game.hit, p.guess),
            ),
        )

        if (
            winner !== undefined &&
            game.players.find((p) => p.id === user?.id) !== undefined &&
            winner.id !== user?.id
        ) {
            EventManager.publish(Events.playSfx, { sfx: Sfx.youLose })
        } else if (winner !== undefined && winner.id === user?.id) {
            EventManager.publish(Events.playSfx, { sfx: Sfx.youWin })
        }
    }, [game])

    return (
        <Modal show={show} onHide={onHide}>
            <Modal.Header closeButton closeLabel={t("close")}>
                <Modal.Title>{t("gameEnded")}</Modal.Title>
            </Modal.Header>
            <Modal.Body>
                <h2 className="h4">
                    {winner !== undefined && winner.id === user?.id
                        ? t("youWin")
                        : winner !== undefined
                          ? t("otherWins", { player: winner.name })
                          : t("nooneWins")}
                </h2>
                <p>{t("finalScore")}</p>
                <Table responsive>
                    <thead>
                        <tr>
                            <th>{t("player", { count: 1 })}</th>
                            <th>{t("token", { count: 2 })}</th>
                            <th>{t("hit", { count: 2 })}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {game.players.map((p) => (
                            <tr>
                                <td>{p.name}</td>
                                <td>{p.tokens}</td>
                                <td>{p.hits.length}</td>
                            </tr>
                        ))}
                    </tbody>
                </Table>
            </Modal.Body>
        </Modal>
    )
}
