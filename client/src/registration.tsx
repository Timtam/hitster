import { useState } from "react"
import Button from "react-bootstrap/Button"
import BsForm from "react-bootstrap/Form"
import { Helmet } from "react-helmet-async"
import { Trans, useTranslation } from "react-i18next"
import type { ActionFunction } from "react-router"
import { Form, Link, useActionData } from "react-router-dom"
import Error from "./error"

export const action: ActionFunction = async ({ request }) => {
    let formData = await request.formData()
    let res = await fetch("/api/users/signup", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            username: formData.get("username"),
            password: formData.get("password"),
        }),
    })

    if (res.status === 200) return { success: true, message: "" }
    return { success: false, message: await res.text() }
}

export function Registration() {
    let response = useActionData() as {
        success: boolean
        message: string
    }
    let [username, setUsername] = useState("")
    let [password, setPassword] = useState("")
    let [passwordRepetition, setPasswordRepetition] = useState("")
    let { t } = useTranslation()

    return (
        <>
            <Helmet>
                <title>{t("register")} - Hitster</title>
            </Helmet>
            {response && response.success === true ? (
                <p>
                    <Trans i18nKey="registrationSuccessful">
                        You've been registered successfully. You can now move on
                        to <Link to="/login">login</Link>
                    </Trans>
                </p>
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
                        <Button
                            variant="primary"
                            type="submit"
                            disabled={
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
