import Modal from "react-bootstrap/Modal"
import Table from "react-bootstrap/Table"
import { useTranslation } from "react-i18next"
import { useContext } from "../../context"
import type { Game, Player } from "../../entities"

export default ({
    game,
    show,
    onHide,
    winner,
}: {
    game: Game
    show: boolean
    onHide: () => void
    winner: Player | null
}) => {
    let { t } = useTranslation()
    let { user } = useContext()

    return (
        <Modal show={show} onHide={onHide}>
            <Modal.Header closeButton closeLabel={t("close")}>
                <Modal.Title>{t("gameEnded")}</Modal.Title>
            </Modal.Header>
            <Modal.Body>
                <h2 className="h4">
                    {winner?.id === user?.id
                        ? t("youWin")
                        : winner !== null
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
                            <tr key={`player-${p.id}`}>
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
