import Modal from "react-bootstrap/Modal"
import { useTranslation } from "react-i18next"
import { Pack } from "../entities"

export default function ViewPacksModal({
    packs,
    onHide,
    show,
}: {
    packs: Pack[]
    onHide: () => void
    show: boolean
}) {
    const { t } = useTranslation()

    return (
        <Modal show={show} onHide={onHide}>
            <Modal.Header closeButton closeLabel={t("close")} />
            <Modal.Body>
                <ul>
                    {packs.map((p) => (
                        <li>{p.name}</li>
                    ))}
                </ul>
            </Modal.Body>
        </Modal>
    )
}
