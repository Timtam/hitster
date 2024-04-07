import { useLocalStorage } from "@uidotdev/usehooks"
import Form from "react-bootstrap/Form"
import Modal from "react-bootstrap/Modal"
import ToggleButton from "react-bootstrap/ToggleButton"
import ToggleButtonGroup from "react-bootstrap/ToggleButtonGroup"
import { useTranslation } from "react-i18next"

export default function Settings({
    show,
    onHide,
}: {
    show: boolean
    onHide: () => void
}) {
    let {
        t,
        i18n: { changeLanguage, language, services },
    } = useTranslation()
    let [musicVolume, setMusicVolume] = useLocalStorage("musicVolume", "1.0")

    return (
        <Modal show={show} onHide={onHide}>
            <Modal.Header closeButton closeLabel={t("close")}>
                <Modal.Title>{t("settings")}</Modal.Title>
            </Modal.Header>
            <Modal.Body>
                <h2 className="h4">{t("volume")}</h2>
                <Form.Label>{t("musicVolume")}</Form.Label>
                <Form.Range
                    min="0"
                    max="100"
                    value={parseFloat(musicVolume) * 100}
                    onChange={(e) =>
                        setMusicVolume(
                            (
                                parseFloat(e.currentTarget.value) / 100
                            ).toString(),
                        )
                    }
                />
                <h2 className="h4">{t("language")}</h2>
                <ToggleButtonGroup
                    name="language"
                    type="radio"
                    value={language}
                    defaultValue={
                        Object.keys(services.resourceStore.data).find(
                            (langcode) => language.startsWith(langcode),
                        ) ?? "en"
                    }
                    onChange={(e) => changeLanguage(e)}
                >
                    {Object.keys(services.resourceStore.data).map(
                        (langcode) => {
                            let nameGenerator = new Intl.DisplayNames(
                                langcode,
                                {
                                    type: "language",
                                },
                            )
                            let displayName = nameGenerator.of(langcode)
                            return (
                                <ToggleButton
                                    className="me-2"
                                    value={langcode}
                                    id={langcode}
                                >
                                    {displayName}
                                </ToggleButton>
                            )
                        },
                    )}
                </ToggleButtonGroup>
            </Modal.Body>
        </Modal>
    )
}
