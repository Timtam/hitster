import { useState } from "react"
import Button from "react-bootstrap/Button"
import BsForm from "react-bootstrap/Form"
import type { ActionFunction } from "react-router"
import { Form, redirect, useActionData } from "react-router-dom"

export const action: ActionFunction = async ({ request }) => {
    let formData = await request.formData()
    let res = await fetch("/api/users/login", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            username: formData.get("username"),
            password: formData.get("password"),
        }),
    })

    if (res.status === 200) return redirect("/")
    return { success: false, message: await res.text() }
}

export function Login() {
    let response = useActionData() as {
        success: boolean
        message: string
    }
    let [username, setUsername] = useState("")
    let [password, setPassword] = useState("")

    return (
        <>
            {response && response.success === false ? (
                <p>
                    An error occurred while logging in:{" "}
                    {JSON.parse(response.message).message}{" "}
                </p>
            ) : (
                ""
            )}
            <Form method="post">
                <BsForm.Group controlId="basicFormUsername">
                    <BsForm.Label>Username</BsForm.Label>
                    <BsForm.Control
                        type="input"
                        name="username"
                        placeholder="Username"
                        value={username}
                        onChange={(evt) => setUsername(evt.target.value)}
                    />
                </BsForm.Group>
                <BsForm.Group controlId="formBasicPassword">
                    <BsForm.Label>Password</BsForm.Label>
                    <BsForm.Control
                        type="password"
                        name="password"
                        placeholder="Password"
                        value={password}
                        onChange={(evt) => setPassword(evt.target.value)}
                    />
                </BsForm.Group>
                <Button
                    variant="primary"
                    type="submit"
                    disabled={username.length === 0 || password.length === 0}
                >
                    Login
                </Button>
            </Form>
        </>
    )
}
