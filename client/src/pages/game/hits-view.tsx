import { useEffect, useRef } from "react"
import Modal from "react-bootstrap/Modal"
import Table from "react-bootstrap/Table"
import { useTranslation } from "react-i18next"
import { Player } from "../../entities"
import { HitPlayer, HitPlayerRef } from "./hit-player"

export default function HitsView({
    show,
    onHide,
    player,
    gameId,
}: {
    show: boolean
    onHide: () => void
    player: Player
    gameId: string
}) {
    const players = useRef<Array<HitPlayerRef | null>>([])
    const { t } = useTranslation()

    useEffect(() => {
        players.current = []
    }, [player])

    return (
        <Modal show={show} onHide={onHide}>
            <Modal.Header closeButton closeLabel={t("close")}>
                <Modal.Title>
                    {t("hitsForPlayer", {
                        player: player.name,
                    })}
                </Modal.Title>
            </Modal.Header>
            <Modal.Body>
                <Table responsive>
                    <thead>
                        <tr>
                            <th>{t("artist")}</th>
                            <th>{t("title")}</th>
                            <th>{t("year")}</th>
                            <th>{t("belongsTo")}</th>
                            <th>
                                {t("pack", {
                                    count: 1,
                                })}
                            </th>
                            <th>{t("playHit")}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {player.hits
                            .toSorted((a, b) => a.year - b.year)
                            .map((h, i) => (
                                <tr key={`hit-${h.id}`}>
                                    <td>{h.artist}</td>
                                    <td>{h.title}</td>
                                    <td>{h.year}</td>
                                    <td>{h.belongs_to}</td>
                                    <td>{h.pack}</td>
                                    <td>
                                        <HitPlayer
                                            src={`/api/games/${gameId}/hit/${h.id}?key=${Math.random()}`}
                                            autoplay={false}
                                            duration={0}
                                            ref={(e) => {
                                                players.current[i] = e
                                            }}
                                            onPlay={() =>
                                                players.current.forEach(
                                                    (p, j) => {
                                                        if (j !== i) p?.stop()
                                                    },
                                                )
                                            }
                                        />
                                    </td>
                                </tr>
                            ))}
                    </tbody>
                </Table>
            </Modal.Body>
        </Modal>
    )
}
