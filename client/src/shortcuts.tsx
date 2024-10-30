import Modal from "react-bootstrap/Modal"
import Table from "react-bootstrap/Table"
import { useTranslation } from "react-i18next"

export default function Shortcuts({
    show,
    onHide,
}: {
    show: boolean
    onHide: () => void
}) {
    let { t } = useTranslation()

    return (
        <Modal show={show} onHide={onHide}>
            <Modal.Header closeButton closeLabel={t("close")}>
                <Modal.Title>{t("keyboardShortcut", { count: 2 })}</Modal.Title>
            </Modal.Header>
            <Modal.Body>
                {show ? (
                    <>
                        <h2>{t("keyboardShortcut", { count: 2 })}</h2>
                        <Table responsive>
                            <thead>
                                <tr>
                                    <th>{t("section")}</th>
                                    <th>{t("action")}</th>
                                    <th>
                                        {t("keyboardShortcut", { count: 1 })}
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr>
                                    <th rowSpan={4}>{t("gameLobby")}</th>
                                </tr>
                                <tr>
                                    <td>{t("publicGame")}</td>
                                    <td>{t("publicGameShortcut")}</td>
                                </tr>
                                <tr>
                                    <td>{t("privateGame")}</td>
                                    <td>{t("privateGameShortcut")}</td>
                                </tr>
                                <tr>
                                    <td>{t("localGame")}</td>
                                    <td>{t("localGameShortcut")}</td>
                                </tr>
                                <tr>
                                    <th rowSpan={22}>{t("game")}</th>
                                </tr>
                                <tr>
                                    <td>{t("joinGame")}</td>
                                    <td>{t("joinGameShortcut")}</td>
                                </tr>
                                <tr>
                                    <td>{t("leaveGame")}</td>
                                    <td>{t("leaveGameShortcut")}</td>
                                </tr>
                                <tr>
                                    <td>{t("startGame")}</td>
                                    <td>{t("startGameShortcut")}</td>
                                </tr>
                                <tr>
                                    <td>{t("stopGame")}</td>
                                    <td>{t("stopGameShortcut")}</td>
                                </tr>
                                <tr>
                                    <td>{t("gameSettings")}</td>
                                    <td>{t("gameSettingsShortcut")}</td>
                                </tr>
                                <tr>
                                    <td>{t("confirmYes")}</td>
                                    <td>{t("yesShortcut")}</td>
                                </tr>
                                <tr>
                                    <td>{t("confirmNo")}</td>
                                    <td>{t("noShortcut")}</td>
                                </tr>
                                <tr>
                                    <td>{t("selectPreviousSlot")}</td>
                                    <td>{t("selectPreviousSlotShortcut")}</td>
                                </tr>
                                <tr>
                                    <td>{t("selectNextSlot")}</td>
                                    <td>{t("selectNextSlotShortcut")}</td>
                                </tr>
                                <tr>
                                    <td>{t("selectNoSlot")}</td>
                                    <td>{t("selectNoSlotShortcut")}</td>
                                </tr>
                                <tr>
                                    <td>{t("submitGuess")}</td>
                                    <td>{t("submitGuessShortcut")}</td>
                                </tr>
                                {Array.from({ length: 10 }, (_, i) => (
                                    <tr>
                                        <td>
                                            {t("speakPlayerInfo", {
                                                player: i + 1,
                                            })}
                                        </td>
                                        <td>
                                            {t("speakPlayerInfoShortcut", {
                                                player: i !== 9 ? i + 1 : 0,
                                            })}
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </Table>
                    </>
                ) : (
                    ""
                )}
            </Modal.Body>
        </Modal>
    )
}
