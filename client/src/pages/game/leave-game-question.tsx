import Button from "react-bootstrap/Button"
import Modal from "react-bootstrap/Modal"
import { useTranslation } from "react-i18next"

export default function LeaveGameQuestion({
    show,
    onHide,
}: {
    show: boolean
    onHide: (yes: boolean) => void
}) {
    let { t } = useTranslation()

    return (
        <Modal show={show} onHide={() => {}}>
            <Modal.Header>
                <Modal.Title>{t("leaveGame")}</Modal.Title>
            </Modal.Header>
            <Modal.Body>
                {show ? (
                    <>
                        <h2>{t("leaveGameQuestion")}</h2>
                        <Button onClick={() => onHide(false)}>{t("no")}</Button>
                        <Button onClick={() => onHide(true)}>{t("yes")}</Button>
                    </>
                ) : (
                    ""
                )}
            </Modal.Body>
        </Modal>
    )
}
