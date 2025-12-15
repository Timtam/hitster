import type { ActionFunction } from "react-router"

const action: ActionFunction = async function ({ request }) {
    const formData = await request.formData()
    const res = await fetch("/api/users/register", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            username: formData.get("username"),
            password: formData.get("password"),
            altcha_token: formData.get("altchaToken"),
        }),
    })

    if (res.status === 200) return { success: true, message: "" }
    return { success: false, message: await res.text() }
}

export default action
