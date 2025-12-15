import { Helmet } from "@dr.pogodin/react-helmet"
import { useEffect, useState } from "react"
import Button from "react-bootstrap/Button"
import BsForm from "react-bootstrap/Form"
import { useTranslation } from "react-i18next"
import { Form, useActionData } from "react-router"
import { useContext } from "../context"
import Error from "../error"

export default function Registration() {
    const { user } = useContext()
    const response = useActionData() as {
        success: boolean
        message: string
    }
    const [username, setUsername] = useState(user?.name ?? "")
    const [password, setPassword] = useState("")
    const [passwordRepetition, setPasswordRepetition] = useState("")
    const { t } = useTranslation()
    let [altchaVerified, setAltchaVerified] = useState(false)

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

    return (
        <>
            <Helmet>
                <title>{t("register")} - Hitster</title>
            </Helmet>
            {response && response.success === true ? (
                <p>{t("registrationSuccessful")}</p>
            ) : (
                <>
                    <Error
                        text={
                            response?.success === false
                                ? JSON.parse(response.message).message
                                : undefined
                        }
                    />
                    <Form method="post">
                        <BsForm.Group controlId="basicFormUsername">
                            <BsForm.Label>{t("username")}</BsForm.Label>
                            <BsForm.Control
                                type="input"
                                name="username"
                                placeholder={t("username")}
                                value={username}
                                onChange={(evt) =>
                                    setUsername(evt.target.value)
                                }
                            />
                        </BsForm.Group>
                        <BsForm.Group controlId="formBasicPassword">
                            <BsForm.Label>{t("password")}</BsForm.Label>
                            <BsForm.Control
                                type="password"
                                name="password"
                                placeholder={t("password")}
                                value={password}
                                onChange={(evt) =>
                                    setPassword(evt.target.value)
                                }
                            />
                        </BsForm.Group>
                        <BsForm.Group controlId="formBasicPasswordRepeat">
                            <BsForm.Label>{t("repeatPassword")}</BsForm.Label>
                            <BsForm.Control
                                type="password"
                                placeholder={t("repeatPassword")}
                                value={passwordRepetition}
                                onChange={(evt) =>
                                    setPasswordRepetition(evt.target.value)
                                }
                            />
                        </BsForm.Group>
                        <altcha-widget
                            challengeurl="/api/altcha"
                            name="altchaToken"
                            auto="onfocus"
                        />
                        <Button
                            variant="primary"
                            type="submit"
                            disabled={
                                !altchaVerified ||
                                username.length === 0 ||
                                password.length === 0 ||
                                password !== passwordRepetition
                            }
                        >
                            {t("register")}
                        </Button>
                    </Form>
                </>
            )}
        </>
    )
}
