import EventManager from "@lomray/event-manager"
import { useEffect, useMemo, useState } from "react"
import Button from "react-bootstrap/Button"
import Form from "react-bootstrap/Form"
import Modal from "react-bootstrap/Modal"
import { useTranslation } from "react-i18next"
import { useContext } from "../context"
import { Events } from "../events"
import HitService from "../services/hits.service"

export default function ReportHitIssueModal({
    show,
    hitId,
    onHide,
    onSubmitted,
}: {
    show: boolean
    hitId: string
    onHide: () => void
    onSubmitted?: () => void
}) {
    const { t } = useTranslation()
    const { user, showError } = useContext()
    const hitService = useMemo(() => new HitService(), [])
    const [message, setMessage] = useState("")
    const [submitting, setSubmitting] = useState(false)
    const [altchaVerified, setAltchaVerified] = useState(false)

    useEffect(() => {
        import("altcha")

        // Listen for Altcha verification
        const interval = setInterval(() => {
            const widget = document.querySelector("altcha-widget")
            if (widget) {
                widget.addEventListener("statechange", (e) => {
                    // @ts-expect-error event type not typed
                    if (e.detail.state === "verified") {
                        setAltchaVerified(true)
                    }
                })
                clearInterval(interval)
            }
        }, 100)

        return () => clearInterval(interval)
    }, [setAltchaVerified])

    useEffect(() => {
        if (!show) return

        EventManager.publish(Events.popup)
        setMessage("")
    }, [show, setMessage])

    const canSubmit =
        user?.permissions.write_issues === true &&
        message.trim().length > 0 &&
        altchaVerified &&
        !submitting

    return (
        <Modal show={show} onHide={onHide}>
            <Modal.Header closeButton closeLabel={t("close")}>
                <Modal.Title>{t("reportIssue")}</Modal.Title>
            </Modal.Header>
            <Modal.Body>
                <Form
                    id="reportIssueForm"
                    onSubmit={(e) => {
                        e.preventDefault()
                    }}
                >
                    <Form.Group className="mb-3">
                        <Form.Label>{t("issueMessageLabel")}</Form.Label>
                        <Form.Control
                            as="textarea"
                            rows={4}
                            value={message}
                            placeholder={t("issueMessagePlaceholder")}
                            onChange={(e) => setMessage(e.currentTarget.value)}
                        />
                    </Form.Group>
                    <altcha-widget
                        challengeurl="/api/altcha"
                        name="altchaToken"
                        auto="onfocus"
                    />
                    <Button
                        disabled={!canSubmit}
                        onClick={async () => {
                            if (!canSubmit) return
                            setSubmitting(true)
                            try {
                                const token =
                                    document.forms.namedItem("reportIssueForm")
                                        ?.altchaToken.value
                                await hitService.createIssue(
                                    hitId,
                                    message.trim(),
                                    token,
                                )
                                setMessage("")
                                onSubmitted?.()
                                onHide()
                            } catch (e) {
                                showError((e as any).message)
                            } finally {
                                setSubmitting(false)
                            }
                        }}
                    >
                        {submitting ? t("submitting") : t("submitIssue")}
                    </Button>
                </Form>
            </Modal.Body>
        </Modal>
    )
}
