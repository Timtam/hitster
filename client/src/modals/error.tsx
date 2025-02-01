import Modal from "react-bootstrap/Modal"
import { useTranslation } from "react-i18next"

export default function ErrorModal({
    error,
    onHide,
}: {
    error?: string
    onHide: () => void
}) {
    const { t } = useTranslation()

    return (
        <Modal show={error !== undefined} onHide={onHide}>
            <Modal.Header closeButton closeLabel={t("close")}>
                <Modal.Title>{t("error")}</Modal.Title>
            </Modal.Header>
            <Modal.Body>
                {error !== undefined ? (
                    <>
                        <h2>{t("error")}</h2>
                        <p>{error}</p>
                    </>
                ) : (
                    ""
                )}
            </Modal.Body>
        </Modal>
    )
}
