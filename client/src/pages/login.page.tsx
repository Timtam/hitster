import { useState } from "react"
import Button from "react-bootstrap/Button"
import BsForm from "react-bootstrap/Form"
import { Helmet } from "react-helmet-async"
import { useTranslation } from "react-i18next"
import { Form, useActionData } from "react-router-dom"
import Error from "../error"

export default function Login() {
    const response = useActionData() as {
        success: boolean
        message: string
    }
    const [username, setUsername] = useState("")
    const [password, setPassword] = useState("")
    const { t } = useTranslation()

    return (
        <>
            <Helmet>
                <title>{t("login")} - Hitster</title>
            </Helmet>
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
                        onChange={(evt) => setUsername(evt.target.value)}
                    />
                </BsForm.Group>
                <BsForm.Group controlId="formBasicPassword">
                    <BsForm.Label>{t("password")}</BsForm.Label>
                    <BsForm.Control
                        type="password"
                        name="password"
                        placeholder={t("password")}
                        value={password}
                        onChange={(evt) => setPassword(evt.target.value)}
                    />
                </BsForm.Group>
                <Button
                    variant="primary"
                    type="submit"
                    disabled={username.length === 0 || password.length === 0}
                >
                    {t("login")}
                </Button>
            </Form>
        </>
    )
}
