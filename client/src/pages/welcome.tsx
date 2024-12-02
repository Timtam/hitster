import sum from "ml-array-sum"
import { useEffect, useState } from "react"
import Modal from "react-bootstrap/Modal"
import Spinner from "react-bootstrap/Spinner"
import { useTranslation } from "react-i18next"
import HitService from "../services/hits.service"

export function Welcome({
    show,
    onHide,
}: {
    show: boolean
    onHide: () => void
}) {
    let { t } = useTranslation()
    let [packs, setPacks] = useState<Record<string, number>>({})

    useEffect(() => {
        if (show) {
            ;(async () => {
                let hs = new HitService()
                let availablePacks = await hs.getAllPacks()
                setPacks(availablePacks)
            })()
        }
    }, [show])

    return (
        <Modal show={show} onHide={onHide}>
            <Modal.Header closeButton closeLabel={t("close")}>
                <Modal.Title>{t("welcome")}</Modal.Title>
            </Modal.Header>
            <Modal.Body>
                {show ? (
                    Object.keys(packs).length === 0 ? (
                        <Spinner animation="border" role="status">
                            <span className="visually-hidden">
                                {t("loading")}
                            </span>
                        </Spinner>
                    ) : (
                        <>
                            <h2>{t("welcome")}</h2>
                            <p>{t("welcomeText")}</p>
                            <h3>{t("features")}</h3>
                            <ul>
                                <li>{t("noRegistrationFeature")}</li>
                                <li>{t("publicAndPrivateGamesFeature")}</li>
                                <li>{t("localGamesFeature")}</li>
                                <li>
                                    {t("packsFeature", {
                                        hits: sum(Object.values(packs)),
                                        packs: Object.keys(packs).length,
                                    })}
                                </li>
                                <li>{t("cardCorrectionFeature")}</li>
                                <li>{t("accessibilityFeature")}</li>
                                <li>{t("colorSchemesFeature")}</li>
                            </ul>
                            <h3>{t("howToPlay")}</h3>
                        </>
                    )
                ) : (
                    ""
                )}
            </Modal.Body>
        </Modal>
    )
}
