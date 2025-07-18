import { useMemo, useState } from "react"
import Button from "react-bootstrap/Button"
import Form from "react-bootstrap/Form"
import Modal from "react-bootstrap/Modal"
import { useTranslation } from "react-i18next"
import type { Game } from "../../entities"
import GameService from "../../services/games.service"

export default function AddLocalPlayer({
    game,
    show,
    onHide,
}: {
    game: Game
    show: boolean
    onHide: () => void
}) {
    const gameService = useMemo(() => new GameService(), [])
    const { t } = useTranslation()
    const [name, setName] = useState("")

    return (
        <Modal show={show} onHide={onHide}>
            <Modal.Header closeButton closeLabel={t("close")}>
                <Modal.Title>{t("addPlayer")}</Modal.Title>
            </Modal.Header>
            <Modal.Body>
                <h2 className="h4">{t("addPlayer")}</h2>
                <Form>
                    <Form.Group
                        className="mb-2"
                        controlId="formLocalPlayerName"
                    >
                        <Form.Label>{t("name")}</Form.Label>
                        <Form.Control
                            type="input"
                            placeholder={t("name")}
                            value={name}
                            onChange={(e) => setName(e.currentTarget.value)}
                        />
                    </Form.Group>
                    <Button
                        type="submit"
                        variant="primary"
                        disabled={name === ""}
                        onClick={async (e) => {
                            e.preventDefault()
                            await gameService.addPlayer(game.id, name)
                            onHide()
                        }}
                    >
                        {t("addPlayer")}
                    </Button>
                </Form>
            </Modal.Body>
        </Modal>
    )
}
