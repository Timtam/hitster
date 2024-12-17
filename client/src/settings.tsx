import { useLocalStorage } from "@uidotdev/usehooks"
import { useEffect } from "react"
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
    let [colorScheme, setColorScheme] = useLocalStorage("colorScheme", "auto")
    let [musicVolume, setMusicVolume] = useLocalStorage("musicVolume", "1.0")
    let [sfxVolume, setSfxVolume] = useLocalStorage("sfxVolume", "1.0")

    useEffect(() => {
        if (!Object.keys(services.resourceStore.data).includes(language)) {
            let lang = Object.keys(services.resourceStore.data).find(
                (langcode) => language.startsWith(langcode),
            )
            if (lang !== undefined) changeLanguage(lang)
        }
    }, [language])

    return (
        <Modal show={show} onHide={onHide}>
            <Modal.Header closeButton closeLabel={t("close")}>
                <Modal.Title>{t("settings")}</Modal.Title>
            </Modal.Header>
            <Modal.Body>
                <h2 className="h4">{t("volume")}</h2>
                <Form>
                    <Form.Group
                        className="mb-2"
                        controlId="settings-music-volume"
                    >
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
                    </Form.Group>
                    <Form.Group
                        className="mb-2"
                        controlId="settings-sfx-volume"
                    >
                        <Form.Label>{t("sfxVolume")}</Form.Label>
                        <Form.Range
                            min="0"
                            max="100"
                            value={parseFloat(sfxVolume) * 100}
                            onChange={(e) =>
                                setSfxVolume(
                                    (
                                        parseFloat(e.currentTarget.value) / 100
                                    ).toString(),
                                )
                            }
                        />
                    </Form.Group>
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
                                        id={`lang-${langcode}`}
                                        key={`lang-${langcode}`}
                                    >
                                        {displayName}
                                    </ToggleButton>
                                )
                            },
                        )}
                    </ToggleButtonGroup>
                    <h2 className="h4">{t("colorScheme")}</h2>
                    <ToggleButtonGroup
                        name="colorScheme"
                        type="radio"
                        value={colorScheme}
                        defaultValue={colorScheme}
                        onChange={(e) => setColorScheme(e)}
                    >
                        <ToggleButton
                            className="me-2"
                            value="auto"
                            id="cs-auto"
                            key="cs-auto"
                        >
                            {t("automatic")}
                        </ToggleButton>
                        <ToggleButton
                            className="me-2"
                            value="light"
                            id="cs-light"
                            key="cs-light"
                        >
                            {t("light")}
                        </ToggleButton>
                        <ToggleButton
                            className="me-2"
                            value="dark"
                            id="cs-dark"
                            key="cs-dark"
                        >
                            {t("dark")}
                        </ToggleButton>
                    </ToggleButtonGroup>
                </Form>
            </Modal.Body>
        </Modal>
    )
}
