import sum from "ml-array-sum"
import { useEffect, useState } from "react"
import Modal from "react-bootstrap/Modal"
import Spinner from "react-bootstrap/Spinner"
import Tab from "react-bootstrap/Tab"
import Tabs from "react-bootstrap/Tabs"
import { Trans, useTranslation } from "react-i18next"
import HitService from "../services/hits.service"

export default function WelcomModale({
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
                                <Trans
                                    components={[<li />]}
                                    values={{
                                        hits: sum(Object.values(packs)),
                                        packs: Object.keys(packs).length,
                                    }}
                                    i18nKey="featuresList"
                                />
                            </ul>
                            <h3>{t("howToPlay")}</h3>
                            <Tabs defaultActiveKey="beginner" className="mb-3">
                                <Tab
                                    eventKey="beginner"
                                    title={t("manualBasicTitle")}
                                >
                                    <Trans
                                        i18nKey="manualBasic"
                                        components={{ ol: <ol />, li: <li /> }}
                                    />
                                </Tab>
                                <Tab
                                    eventKey="advanced"
                                    title={t("manualAdvancedTitle")}
                                >
                                    {" "}
                                    <Trans
                                        i18nKey="manualAdvanced"
                                        components={{ ul: <ul />, li: <li /> }}
                                    />
                                </Tab>
                            </Tabs>
                        </>
                    )
                ) : (
                    ""
                )}
            </Modal.Body>
        </Modal>
    )
}
